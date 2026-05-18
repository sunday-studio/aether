pub mod groq;
pub mod local_whisper;
pub mod openai;
pub mod self_hosted;

pub use groq::GroqProvider;
pub use local_whisper::LocalWhisperProvider;
pub use openai::OpenAIProvider;
pub use self_hosted::SelfHostedProvider;
