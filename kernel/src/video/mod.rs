// TrustVideo — custom video codec & player for TrustOS
// Format: .tv (TrustVideo) — delta-encoded frames with RLE compression
// No external dependencies, no graphics API — pure software rendering

pub mod codec;
pub mod player;

pub use codec::{TvHeader, TvEncoder, TvDecoder};
pub use player::VideoPlayer;
