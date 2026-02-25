use thirtyfour::prelude::*;

#[tokio::main]
async fn main() -> WebDriverResult<()> {

    let driver = WebDriver::new("http://localhost:56118", caps).await?;

    driver.goto("https://wikipedia.org").await?;
    println!("Открыта страница: {}", driver.title().await?);

    let search_form = driver.find(By::Id("search-form")).await?;

    let search_input = search_form.find(By::Id("searchInput")).await?;

    search_input.send_keys("Rust programming language").await?;

    let search_button = search_form.find(By::Css("button[type='submit']")).await?;
    search_button.click().await?;

    let first_heading = driver.find(By::ClassName("firstHeading")).await?;
    println!("Заголовок страницы: {}", first_heading.text().await?);

    assert_eq!(driver.title().await?, "Rust programming language - Wikipedia");

    driver.quit().await?;

    Ok(())
}