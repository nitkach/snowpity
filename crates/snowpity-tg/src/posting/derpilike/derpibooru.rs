use self::derpitools::Derpitools;
use crate::posting::derpilike::api::MediaId;
use crate::posting::derpilike::*;
use crate::posting::platform::ParsedQuery;
use crate::Result;
use async_trait::async_trait;

pub(crate) struct Platform {
    tools: Derpitools,
}

impl PlatformTypes for Platform {
    type PostId = MediaId;
    type BlobId = ();
    type Request = MediaId;
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

    fn parse_query(query: &str) -> Option<ParsedQuery<Self>> {
        let (_, origin, host, id) = parse_with_regexes!(
            query,
            r"((trixiebooru.org|derpibooru.org)(?:/images)?)/(\d+)",
            r"(()derpicdn.net/img)/\d+/\d+/\d+/(\d+)",
            r"(()derpicdn.net/img/(?:view|download))/\d+/\d+/\d+/(\d+)",
        )?;

        let mirror = Mirror::if_differs(host, "derpibooru.org");

        ParsedQuery::builder()
            .origin(origin)
            .mirror(mirror)
            .request(id.parse().ok()?)
            .build()
            .into()
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
