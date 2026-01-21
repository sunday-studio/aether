pub mod openai;
pub mod groq;
pub mod local_whisper;
pub mod self_hosted;

pub use openai::OpenAIProvider;
pub use groq::GroqProvider;
pub use local_whisper::LocalWhisperProvider;
pub use self_hosted::SelfHostedProvider;
