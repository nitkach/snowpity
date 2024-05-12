use async_trait::async_trait;

use crate::posting::derpilike::api::MediaId;
use crate::posting::derpilike::*;
use crate::Result;

use self::derpitools::Derpitools;

pub(crate) struct Platform {
    tools: Derpitools,
}

impl PlatformTypes for Platform {
    type PostId = MediaId;
    type BlobId = ();
    type RequestId = MediaId;
}

#[async_trait]
impl PlatformTrait for Platform {
    type Config = Config;

    const NAME: &'static str = "Derpibooru";

    fn new(params: PlatformParams<Config>) -> Self {
        Self {
            tools: Derpitools::new(params, DerpiPlatformKind::Derpibooru),
        }
    }

    fn parse_query(query: &str) -> ParseQueryResult<MediaId> {
        let (_, host, id) = parse_with_regexes!(
            query,
            r"(derpibooru.org(?:/images)?)/(\d+)",
            r"(trixiebooru.org(?:/images)?)/(\d+)",
            r"(derpicdn.net/img)/\d+/\d+/\d+/(\d+)",
            r"(derpicdn.net/img/(?:view|download))/\d+/\d+/\d+/(\d+)",
        )?;
        Some((host.into(), id.parse().ok()?))
    }

    async fn get_post(&self, media: MediaId) -> Result<Post<Self>> {
        self.tools.get_post(media).await
    }

    async fn get_cached_blobs(&self, media: MediaId) -> Result<Vec<CachedBlobId<Self>>> {
        self.tools.get_cached_blobs(media).await
    }

    async fn set_cached_blob(&self, media: MediaId, blob: CachedBlobId<Self>) -> Result {
        self.tools.set_cached_blob(media, blob).await
    }
}
