use cookie_scoop::{GetCookiesOptions, BrowserName};

#[tokio::main]
async fn main() {
    let url = "https://sitor.cc";
    println!("Testing cookie_scoop for URL: {}", url);
    println!("=====================================\n");

    let opts = GetCookiesOptions::new(url)
        .browsers(vec![BrowserName::Chrome])
        .timeout_ms(5000);

    let result = cookie_scoop::get_cookies(opts).await;

    println!("Total cookies found: {}", result.cookies.len());
    println!("Warnings: {:?}", result.warnings);
    println!();

    for (i, cookie) in result.cookies.iter().enumerate() {
        println!("Cookie #{}:", i + 1);
        println!("  Name: {}", cookie.name);
        println!("  Value: {}", if cookie.value.len() > 50 { &cookie.value[..50] } else { &cookie.value });
        println!("  Domain: {:?}", cookie.domain);
        println!("  Path: {:?}", cookie.path);
        println!("  Secure: {:?}", cookie.secure);
        println!("  HttpOnly: {:?}", cookie.http_only);
        println!("  SameSite: {:?}", cookie.same_site);
        println!("  Expires: {:?}", cookie.expires);
        println!("  Source: {:?}", cookie.source);
        println!();
    }
}
