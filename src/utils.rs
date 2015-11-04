/// Remove duplicate and trailing slashes from a path
pub fn squeeze_path(pth: String) -> String {
    let mut st = String::new();
    for p in pth.split('/').filter(|&x| x != "") {
        st.push('/');
        st.push_str(p)
    }
    if st.len() == 0 {
        String::from("/")
    } else {
        st
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
