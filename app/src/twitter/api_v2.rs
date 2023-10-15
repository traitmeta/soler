use super::*;

pub fn get_bearer_token(bear_token: &str) -> BearerToken {
    BearerToken::new(bear_token)
}

pub fn get_oauth2_token(stored_oauth2_token: &str) -> Oauth2Token {
    serde_json::from_str(stored_oauth2_token).unwrap()
}

pub async fn get_twitter_info(auth: impl Authorization) -> twitter_v2::Tweet {
    TwitterApi::new(auth)
        .get_tweet(1261326399320715264)
        .tweet_fields([TweetField::AuthorId, TweetField::CreatedAt])
        .send()
        .await
        .unwrap()
        .into_data()
        .expect("this tweet should exist")
}

pub async fn get_my_twitter_followers(auth: impl Authorization) -> Option<Vec<twitter_v2::User>> {
    TwitterApi::new(auth)
        .with_user_ctx()
        .await
        .unwrap()
        .get_my_followers()
        .user_fields([UserField::Username])
        .max_results(20)
        .send()
        .await
        .unwrap()
        .into_data()
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    #[ignore]
    fn test_get_twitter_info() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let auth = get_bearer_token("%3DIv4hBRBM6QB5qJdBCjVeIsY1jOmbB3uRPocAoCzWQqDIyltcDp");
        let tweet = rt.block_on(get_twitter_info(auth));
        assert_eq!(tweet.id, 1261326399320715264);
        assert_eq!(tweet.author_id.unwrap(), 2244994945);
        assert_eq!(
            tweet.created_at.unwrap(),
            datetime!(2020-05-15 16:03:42 UTC)
        );
    }
}
