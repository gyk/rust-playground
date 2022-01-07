
#[cfg(test)]
mod tests {
    use globset::*;

    #[test]
    fn file() -> Result<(), Error> {
        let glob = GlobBuilder::new("**/[Dd]esktop.ini")
            .case_insensitive(false)
            .build()?.compile_matcher();

        assert!(glob.is_match("desktop.ini"));
        assert!(glob.is_match("Desktop.ini"));
        assert!(glob.is_match(r"C:\Users\gyk\Pictures\desktop.ini"));
        assert!(glob.is_match("/usr/me/desktop.ini"));

        assert!(!glob.is_match("deskTop.ini"));
        assert!(!glob.is_match("Desktop.ini.txt"));
        assert!(!glob.is_match(r"C:\desktop.ini\inner.txt"));
        Ok(())
    }

    #[test]
    fn folder() -> Result<(), Error> {
        let glob = GlobBuilder::new("**/.AppleDouble/**")
            .build()?.compile_matcher();

        assert!(glob.is_match(".AppleDouble"));
        assert!(glob.is_match(".AppleDouble/"));
        assert!(glob.is_match("/.AppleDouble"));
        assert!(glob.is_match(".AppleDouble/hello"));
        assert!(glob.is_match("/usr/me/.AppleDouble"));
        assert!(glob.is_match("/usr/me/.AppleDouble/"));
        assert!(glob.is_match("/usr/me/.AppleDouble/inner"));

        assert!(!glob.is_match(".AppleDoubleSeed"));

        Ok(())
    }

    #[test]
    fn suffix() -> Result<(), Error> {
        // "**/target/**" doesn't work either
        let glob = GlobBuilder::new("**/target{,/**}")
            .build()?
            .compile_matcher();

        assert!(glob.is_match("target"));
        assert!(glob.is_match("/target"));
        assert!(glob.is_match("target/"));
        assert!(glob.is_match("/path/to/target"));
        assert!(glob.is_match("/path/to/target/"));
        assert!(glob.is_match("/path/to/target/folder"));
        assert!(glob.is_match("/path/to/target/folder/"));

        assert!(!glob.is_match("targetfolder"));

        Ok(())
    }
}
