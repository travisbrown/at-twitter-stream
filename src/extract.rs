use serde_json::{Deserializer, Result, Value};

/// Basic user info extraction (ID, screen name, display name)
pub fn extract_user_info<R: std::io::Read>(
    reader: R,
) -> impl Iterator<Item = Result<Vec<(u64, String, String)>>> {
    Deserializer::from_reader(reader)
        .into_iter::<Value>()
        .map(|res| {
            res.map(|obj| {
                let mut users = Vec::with_capacity(1);

                add_status_users(&mut users, &obj, false);

                users
            })
        })
}

fn add_status_users(users: &mut Vec<(u64, String, String)>, obj: &Value, is_retweeted: bool) {
    if let Some(info) = obj.get("user").and_then(extract_user) {
        users.push(info);
    }

    if is_retweeted {
        if let Some(user_mentions) = obj
            .get("entities")
            .and_then(|obj| obj.get("user_mentions").and_then(|obj| obj.as_array()))
        {
            for user_mention in user_mentions {
                if let Some(info) = extract_user(user_mention) {
                    users.push(info);
                }
            }
        }
    } else if let Some(retweeted) = obj.get("retweeted_status") {
        add_status_users(users, retweeted, true);
    }
}

fn extract_user(obj: &Value) -> Option<(u64, String, String)> {
    let id = obj.get("id_str")?.as_str()?.parse::<u64>().ok()?;
    let screen_name = obj.get("screen_name")?.as_str()?.to_string();
    let name = obj.get("name")?.as_str()?.to_string();

    Some((id, screen_name, name))
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    #[test]
    fn extract_user_info() {
        let test_file = File::open("examples/test-01.json").unwrap();
        let mut user_info = super::extract_user_info(test_file)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(user_info.len(), 10);

        let user_info_batch = user_info.pop().unwrap();
        let expected: Vec<(u64, String, String)> = vec![(
            860973241569071104,
            "gorgartweets".to_string(),
            "GORGAR".to_string(),
        )];

        assert_eq!(user_info_batch, expected);
    }
}
