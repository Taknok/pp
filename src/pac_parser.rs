use crate::PAC_UTILS;

use js_sandbox::Script;

const URL: &str = "https://gist.githubusercontent.com/jupierce/f0f2cd8721c3fa4fbf08451e1003a6e6/raw/9b1ff480e17ac5b56c8b1c6d746d1372069b50b3/proxy.pac";

pub struct PACParser {
  script: js_sandbox::Script,
}

unsafe impl Send for PACParser {}

impl PACParser {
  pub async fn new() -> Self {
    let pac_content = Self::download_pac().await.expect("Error while downloading PAC");
    let source = format!("{}\n{}", PAC_UTILS, pac_content);

    let script = Script::from_string(source.as_str()).unwrap();

    PACParser {
      script: script,
    }
  }

  async fn download_pac() -> Result<String, Box<dyn std::error::Error>> {
    let response = reqwest::get(URL).await?;
    let body = response.bytes().await?;
    let content = String::from_utf8(body.to_vec())?;

    Ok(content)
  }

  pub fn find(&mut self, url: &String, host: &String) -> String {
    let result: String = self.script.call("FindProxyForURL", (url.as_str(), host.as_str())).unwrap();

    result
  }
}
