use super::derpilike::{derpibooru, furbooru, manebooru, ponerpics, ponybooru, twibooru};
use super::platform::prelude::*;
use super::{deviant_art, twitter};
use crate::prelude::*;
use crate::Result;
use assert_matches::assert_matches;
use std::fmt;
use url::Url;

macro_rules! def_all_platforms {
    (
        $([$platform:ident, $Platform:ident]),* $(,)?
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub(crate) enum Request {
            $( $Platform(<$platform::Platform as PlatformTypes>::Request), )*
        }

        #[derive(Clone, PartialEq, Eq, Hash, Debug)]
        pub(crate) enum PostId {
            $( $Platform(<$platform::Platform as PlatformTypes>::PostId), )*
        }

        #[derive(Clone, PartialEq, Eq, Hash, Debug)]
        pub(crate) enum BlobId {
            $( $Platform(<$platform::Platform as PlatformTypes>::BlobId), )*
        }

        #[derive(Clone, PartialEq, Eq, Hash, Debug)]
        pub(crate) enum Mirror {
            $( $Platform(<$platform::Platform as PlatformTypes>::Mirror), )*
        }

        impl fmt::Display for Mirror {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $( Self::$Platform(mirror) => fmt::Display::fmt(mirror, f), )*
                }
            }
        }

        impl MirrorTrait for Mirror {
            fn try_update_url_to_mirror(&self, url: &mut Url) -> Result<(), url::ParseError> {
                match self {
                    $( Self::$Platform(mirror) => mirror.try_update_url_to_mirror(url), )*
                }
            }
        }

        impl DisplayInFileName for PostId {
            fn display_in_file_name(&self) -> Option<String> {
                match self {
                    $( Self::$Platform(id) => id.display_in_file_name(), )*
                }
            }
        }

        impl DisplayInFileName for BlobId {
            fn display_in_file_name(&self) -> Option<String> {
                match self {
                    $( Self::$Platform(id) => id.display_in_file_name(), )*
                }
            }
        }

        impl PostId {
            /// Name of the posting platform that hosts the post.
            pub(crate) fn platform_name(&self) -> &'static str {
                match self {
                    $( Self::$Platform(_) => <$platform::Platform as PlatformTrait>::NAME, )*
                }
            }
        }

        pub(crate) struct Config {
            $( pub(crate) $platform: <$platform::Platform as PlatformTrait>::Config, )*
        }

        impl Config {
            pub(crate) fn load_or_panic() -> Config {
                Self {
                    $(
                        $platform: crate::config::from_env_or_panic(
                            <$platform::Platform as PlatformTrait>::Config::ENV_PREFIX
                        ),
                    )*
                }
            }
        }

        pub(crate) struct AllPlatforms {
            $( $platform: $platform::Platform, )*
        }

        impl AllPlatforms {
            pub(crate) fn new(params: PlatformParams<Config>) -> Self {
                Self {
                    $(
                        $platform: <$platform::Platform as PlatformTrait>::new(PlatformParams {
                            config: params.config.$platform,
                            http: params.http.clone(),
                            db: params.db.clone(),
                        }),
                    )*
                }
            }

            pub(crate) async fn get_post(&self, request: Request) -> Result<Post> {
                Ok(match request {
                    $(
                        Request::$Platform(request) => {
                            let post = self.$platform.get_post(request).await?;
                            let blobs = post.blobs.map_collect(|blob| {
                                let MultiBlob { repr, id } = blob;
                                MultiBlob { repr, id: BlobId::$Platform(id) }
                            });

                            let BasePost {
                                id,
                                authors,
                                web_url,
                                safety,
                            } = post.base;

                            let base = BasePost {
                                id: PostId::$Platform(id),
                                authors,
                                web_url,
                                safety,
                            };

                            Post { base, blobs }
                        }
                    )*
                })
            }

            pub(crate) async fn get_cached_blobs(
                &self,
                request: Request,
            ) -> Result<Vec<CachedBlobId>> {
                Ok(match request {
                    $(
                        Request::$Platform(request) => {
                            self
                                .$platform
                                .get_cached_blobs(request)
                                .await?
                                .map_collect(|blob| CachedBlobId {
                                    id: BlobId::$Platform(blob.id),
                                    tg_file: blob.tg_file,
                                })
                        }
                    )*
                })
            }

            pub(crate) async fn set_cached_blob(
                &self,
                post: PostId,
                blob: CachedBlobId<Self>,
            ) -> Result {
                match post {
                    $(
                        PostId::$Platform(post) => {
                            let id = assert_matches!(blob.id, BlobId::$Platform(blob) => blob);
                            let blob = CachedBlobId {
                                id,
                                tg_file: blob.tg_file,
                            };
                            self.$platform.set_cached_blob(post, blob).await
                        }
                    )*
                }
            }
        }

        pub(crate) fn parse_query(input: &str) -> ParseQueryResult<AllPlatforms> {
            let input = input.trim();

            $(
                if let Some((platform, id)) = <$platform::Platform as PlatformTrait>::parse_query(input) {
                    return Some((platform, Request::$Platform(id)));
                }
            )*

            None
        }
    }
}

def_all_platforms! {
    [derpibooru, Derpibooru],
    [furbooru, Furbooru],
    [manebooru, Manebooru],
    [ponerpics, Ponerpics],
    [ponybooru, Ponybooru],
    [twibooru, Twibooru],
    [twitter, Twitter],
    [deviant_art, DeviantArt],
}

impl PlatformTypes for AllPlatforms {
    type Request = Request;
    type PostId = PostId;
    type BlobId = BlobId;
    type Mirror = Mirror;
}
