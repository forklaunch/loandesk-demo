/// One error the front end caught in a user's browser. It deliberately has no
/// field for the error message: messages can quote what a user typed.
#[forklaunch_core::contract::schema(unknown_fields = "deny")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserErrorReport {
    /// "error" (uncaught exception), "unhandledrejection" or "render".
    #[field(max_length = 32)]
    pub kind: String,
    /// Error class, for example "TypeError".
    #[field(max_length = 80)]
    pub class: String,
    /// Screen it happened on, with record ids replaced, for example "#/applications/:id".
    #[field(max_length = 120)]
    pub screen: String,
    /// Stack frames only: function, script file, line and column.
    #[field(max_length = 4000)]
    pub stack: String,
    /// Front-end build that reported it.
    #[field(max_length = 64)]
    pub release: String,
}
