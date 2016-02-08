use api_client::{Error, Response};
use serde_json;
use serde_json::Value;

pub fn expand_string(val: &Option<String>) -> String {
    match *val {
        None => "".to_owned(),
        Some(ref x) => x.to_owned(),
    }
}

/// Remove duplicate and trailing slashes from a path
pub fn squeeze_path(pth: String) -> String {
    let mut st = String::new();
    for p in pth.split('/').filter(|&x| x != "") {
        st.push('/');
        st.push_str(p)
    }
    if st.is_empty() {
        String::from("/")
    } else {
        st
    }
}

/// The Chef Server returns lists of objects in the form
/// [ "name", "https://chef.local/type/name"]
/// In general, we don't care about the URL, so just give return
/// a list of the names
pub fn decode_list(r: Response) -> Result<Vec<String>, Error> {
    let data: Result<Value, _> = serde_json::from_str(&*r.body);
    match data {
        Ok(data) => {
            if let Some(obj) = data.as_object() {
                let mut resp = vec![];
                for (key, _) in obj.iter() {
                    resp.push(key.clone());
                }
                Ok(resp)
            } else {
                Err(Error::Generic)
            }
        }
        Err(e) => Err(Error::Json(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::squeeze_path;

    #[test]
    fn test_squeeze_path() {
        let path = String::from("/any/given/path");
        assert_eq!(path, squeeze_path(path.clone()))
    }

    #[test]
    fn test_squeeze_path_with_duplicate_slashes() {
        let path = String::from("/any//given/path");
        assert_eq!("/any/given/path", squeeze_path(path.clone()))
    }

    #[test]
    fn test_squeeze_path_with_many_duplicate_slashes() {
        let path = String::from("/any//old/given//path");
        assert_eq!("/any/old/given/path", squeeze_path(path.clone()))
    }

    #[test]
    fn test_squeeze_path_with_trailing_slash() {
        let path = String::from("/any/given/path/");
        assert_eq!("/any/given/path", squeeze_path(path.clone()))
    }
}
