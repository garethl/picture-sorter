use anyhow::Result;

#[derive(Clone, Copy)]
pub struct Cache {
    //
}

impl Cache {
    pub fn new(cache_dir: String) -> Self {
        Cache {
            //
        }
    }

    pub fn get<P>(&self, path: &str, f: impl FnOnce() -> Result<P>) -> Result<P>  {
        f()
    }
}