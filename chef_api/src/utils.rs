use failure::Error;
use errors::ChefError;
use serde_json::Value;

pub fn expand_string(val: &Option<String>) -> String {
    match *val {
        None => "".to_owned(),
        Some(ref x) => x.to_owned(),
    }
}

/// Remove duplicate and trailing slashes from a path
pub fn squeeze_path(pth: &str) -> String {
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

pub fn add_path_element(path: String, elem: &str) -> String {
    let mut path = path;
    path.push('/');
    path.push_str(elem);
    path
}

/// The Chef Server returns lists of objects in the form
/// [ "name", `"https://chef.local/type/name"`]
/// In general, we don't care about the URL, so just return
/// a list of the names
pub fn decode_list(data: &Value) -> Result<Vec<String>, Error> {
    if let Some(obj) = data.as_object() {
        let mut resp = vec![];
        for (key, _) in obj.iter() {
            resp.push(key.clone());
        }
        Ok(resp)
    } else {
        Err(ChefError::ListError.into())
    }
}

#[cfg(test)]
mod tests {
    use super::squeeze_path;

    #[test]
    fn test_squeeze_path() {
        let path = String::from("/any/given/path");
        assert_eq!(path, squeeze_path(&path))
    }

    #[test]
    fn test_squeeze_path_with_duplicate_slashes() {
        let path = String::from("/any//given/path");
        assert_eq!("/any/given/path", squeeze_path(&path))
    }

    #[test]
    fn test_squeeze_path_with_many_duplicate_slashes() {
        let path = String::from("/any//old/given//path");
        assert_eq!("/any/old/given/path", squeeze_path(&path))
    }

    #[test]
    fn test_squeeze_path_with_trailing_slash() {
        let path = String::from("/any/given/path/");
        assert_eq!("/any/given/path", squeeze_path(&path))
    }
}
