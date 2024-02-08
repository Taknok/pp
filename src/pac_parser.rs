use crate::PAC_UTILS;

const URL: &str = "https://gist.githubusercontent.com/jupierce/f0f2cd8721c3fa4fbf08451e1003a6e6/raw/9b1ff480e17ac5b56c8b1c6d746d1372069b50b3/proxy.pac";

pub struct PACParser<'s> {
  platform: v8::UniqueRef<v8::Platform>,
  func: v8::Function,
  scope: v8::HandleScope<'s>,
  global: v8::Local<'s, v8::Object>,
}

impl PACParser<'_> {
  pub async fn new() -> Self {
    let platform = v8::new_default_platform(0, false).make_shared();
    // Initialize V8.
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    // Create a new Isolate and make it the current one.
    let isolate = &mut v8::Isolate::new(v8::CreateParams::default());

    // Create a stack-allocated handle scope.
    let handle_scope = &mut v8::HandleScope::new(isolate);

    // Create a new context.
    let context = v8::Context::new(handle_scope);

    // Enter the context for compiling and running the hello world script.
    let mut scope = &mut v8::ContextScope::new(handle_scope, context);

    let pac_content = Self::download_pac().await.expect("Error while downloading PAC");

    let source = format!("{}\n{}", PAC_UTILS, pac_content);

    // Create a string containing the JavaScript source code.
    let code = v8::String::new(scope, &source).unwrap();

    // Compile the source code.
    let script = v8::Script::compile(scope, code, None).unwrap();
    // Run the script to get the result.
    let _ = script.run(scope).unwrap();

    let f_name = v8::String::new(scope, "FindProxyForURL").unwrap();

    let global = context.global(&mut scope);
    let process_fn = global
      .get(&mut scope, f_name.into())
      .unwrap();

    let func = v8::Local::<v8::Function>::try_from(process_fn)
      .expect("function expected");

    if !func.is_function() {
      panic!("'FindProxyForURL' is not a function");
    }

    PACParser {
      platform: platform,
      func: func,
      scope: scope,
      global: global,
    }
  }

  async fn download_pac() -> Result<String, Box<dyn std::error::Error>> {
    let response = reqwest::get(URL).await?;
    let body = response.bytes().await?;
    let content = String::from_utf8(body.to_vec())?;

    Ok(content)
  }

  pub fn find(&self, url: &String, host: &String) -> String {
    let url_v8 = v8::String::new(&self.scope, url).unwrap();
    let host_v8 = v8::String::new(&self.scope, host).unwrap();
    let result = self.func.call(&self.scope, self.global.into(), &[url_v8.into(), host_v8.into()]).unwrap();
    let result = result.to_string(&self.scope).unwrap();

    result.to_rust_string_lossy(&self.scope)
  }
}
