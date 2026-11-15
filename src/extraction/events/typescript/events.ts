// Generated from the worker's Rust event contract.
export type WorkEvent = { context: { operationId: string; resourceId: string; revision: number; tenantId: string }; input: WorkInput; queue: string };
export type WorkEventInput = WorkInput;
export type WorkInput = { message: string; resourceId: string; revision: number };
const inputSchema = {"$schema":"https://json-schema.org/draft/2020-12/schema","title":"WorkInput","type":"object","properties":{"resourceId":{"type":"string","format":"uuid"},"revision":{"type":"integer","format":"int64","minimum":1,"maximum":9007199254740991},"message":{"type":"string"}},"additionalProperties":false,"required":["resourceId","revision","message"],"x-forklaunch-model":{"version":1,"id":"extraction_contracts::jobs::WorkInput","name":"WorkInput","projection":"value"}};
const eventSchema = {"$schema":"https://json-schema.org/draft/2020-12/schema","title":"WorkEvent","type":"object","properties":{"context":{"$ref":"#/$defs/JobContext"},"queue":{"type":"string"},"input":{"$ref":"#/$defs/WorkInput"}},"additionalProperties":false,"required":["context","queue","input"],"x-forklaunch-model":{"version":1,"id":"extraction_contracts::jobs::WorkEvent","name":"WorkEvent","projection":"value"},"$defs":{"JobContext":{"description":"Public routing metadata, checked against the authenticated inner payload.\nRevision is a positive application version, not the queue's retry counter.","type":"object","properties":{"tenantId":{"type":"string","format":"uuid"},"resourceId":{"type":"string","format":"uuid"},"operationId":{"type":"string","format":"uuid"},"revision":{"type":"integer","format":"int64","minimum":1,"maximum":9007199254740991}},"additionalProperties":false,"required":["tenantId","resourceId","operationId","revision"]},"WorkInput":{"type":"object","properties":{"resourceId":{"type":"string","format":"uuid"},"revision":{"type":"integer","format":"int64","minimum":1,"maximum":9007199254740991},"message":{"type":"string"}},"additionalProperties":false,"required":["resourceId","revision","message"],"x-forklaunch-model":{"version":1,"id":"extraction_contracts::jobs::WorkInput","name":"WorkInput","projection":"value"}}}};

import {Ajv, type Plugin} from 'ajv';
import addFormats from 'ajv-formats';
import {createAuditedWorkConsumer} from './encrypted-worker.mjs';
import type {ConsumerOptions, ChannelContext} from './encrypted-channel.mjs';
export type {ConsumerOptions, ChannelContext, AuditRecord, ChannelKey} from './encrypted-channel.mjs';
export {createEncryptedWorkPublisher} from './encrypted-worker.mjs';
const validator = new Ajv({strict: true, allErrors: false});
(addFormats as unknown as Plugin<unknown>)(validator);
for (const bits of [8,16,32,64]) {
  for (const unsigned of [false,true]) {
    const max = Math.min(Number.MAX_SAFE_INTEGER, 2 ** (unsigned ? bits : bits-1) - 1);
    const min = unsigned ? 0 : Math.max(Number.MIN_SAFE_INTEGER, -(2 ** (bits-1)));
    validator.addFormat(`${unsigned ? 'uint' : 'int'}${bits}`, {type:'number', validate:(value:number)=>Number.isSafeInteger(value) && value >= min && value <= max});
  }
}
const validateInput = validator.compile<WorkInput>(inputSchema);
const validateEvent = validator.compile<WorkEvent>(eventSchema);
/** Adapter must seal with the configured trusted tenant, key and work-v1 channel,
 * bind queue and operation identity, and publish only the resulting SealedEvent.
 * A raw broker client does not implement this interface. */
export interface EncryptedWorkPublisher {
  sealAndPublish(request: { readonly channel: "work-v1"; readonly queue: string;
    readonly operationId: string; readonly input: WorkInput }): Promise<void>;
}
export class WorkProducer {
  constructor(private readonly publisher: EncryptedWorkPublisher, private readonly queue: string) {}
  async publish(operationId: string, input: WorkInput): Promise<void> {
    let resourceId:unknown, revision:unknown, snapshot:unknown;
    try {
      resourceId=input.resourceId; revision=input.revision;
      const serialized=JSON.stringify(input);
      if (new TextEncoder().encode(serialized).length > 1048576) throw new Error();
      snapshot=JSON.parse(serialized);
    } catch { throw new Error("Worker event contract rejected"); }
    if (!validateInput(snapshot) || snapshot.resourceId !== resourceId || snapshot.revision !== revision) throw new Error("Worker event contract rejected");
    await this.publisher.sealAndPublish({channel: "work-v1", queue: this.queue, operationId, input:snapshot});
  }
}
/** The trusted context must come from authorization/dispatch state, not the envelope.
 * Audit must acknowledge durable metadata-only storage before plaintext is returned. */
export class WorkConsumer {
  private readonly consumer: ReturnType<typeof createAuditedWorkConsumer<WorkEvent>>;
  constructor(options: ConsumerOptions, queue: string) {
    this.consumer=createAuditedWorkConsumer(options,queue,validateEvent);
  }
  consume(expected: ChannelContext, envelope: unknown): Promise<WorkEvent> {
    return this.consumer.consume(envelope,expected);
  }
  close():void {this.consumer.close();}
}
