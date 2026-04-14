use anyhow::{Context, Result};
use thirtyfour::prelude::*;
use thirtyfour::By;
use tokio::time::{sleep, Duration};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use thirtyfour::Key;

fn read_users(file_path: &str) -> Result<Vec<(u32, String)>>{
    let file = File::open(file_path).context("unable to open file USERS")?;
    let reader  = BufReader::new(file);
    let mut users = Vec::new();

    for line in reader.lines(){
        let line__ = line?;
        if line__.trim().is_empty(){
            continue;
        }
        let space_idx = line__.find(' ').unwrap_or(line__.len());
        let id_str = &line__[..space_idx];
        let name = if space_idx < line__.len() {
            line__[space_idx + 1..].to_string()
        } else {
            String::new()
        };
        let id: u32 = id_str.parse().context("Wrong format ID")?;
        users.push((id, name));
    }
    Ok(users)
}

fn append_to_file(fila_name: &str, data: &str) -> Result<()>{
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(fila_name)?;
    file.write_all(data.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}


fn is_good_android(user_name: &str) -> bool {
    let filename = format!("{}_history.html", user_name);
    if let Ok(html) = fs::read_to_string(&filename){
        html.contains("iOS") || html.contains("Android")
    } else {
        false
    }
}
async fn click_safety_in_iframe(driver: &WebDriver) -> Result<()> {
    // Ждём появления iframe (не обязательно, но может помочь)
    sleep(Duration::from_secs(2)).await;
    let iframes = driver.find_all(By::Tag("iframe")).await?;
    for (idx, iframe) in iframes.iter().enumerate() {
        // Пытаемся переключиться в iframe
        if let Ok(_) = driver.enter_frame(idx as u16).await {
            // Ищем кнопку внутри фрейма
            let elements = driver.find_all(By::ClassName("ui-btn-text-inner")).await?;
            for el in elements {
                let text = el.text().await.unwrap_or_default();
                if text.contains("Безопасность") {
                    el.click().await?;
                    driver.enter_default_frame().await?;
                    return Ok(());
                }
            }
            // Если кнопка не найдена, выходим из фрейма и продолжаем поиск
            driver.enter_default_frame().await?;
        }
    }
    // Если ни один iframe не подошёл, возвращаем ошибку (или просто Ok(()), если не критично)
    anyhow::bail!("Не найден iframe с кнопкой 'Безопасность'");
}

/// Ищет во всех iframe ссылку "История входов" и кликает по ней.
async fn click_history_in_iframe(driver: &WebDriver) -> Result<()> {
    sleep(Duration::from_secs(2)).await;
    let iframes = driver.find_all(By::Tag("iframe")).await?;
    for (idx, iframe) in iframes.iter().enumerate() {
        if let Ok(_) = driver.enter_frame(idx as u16).await {
            let elements = driver
                .find_all(By::XPath("//div[@class='ui-sidepanel-menu-link-text' and text()='История входов']"))
                .await?;
            if !elements.is_empty() {
                elements[0].click().await?;
                driver.enter_default_frame().await?;
                return Ok(());
            }
            driver.enter_default_frame().await?;
        }
    }
    anyhow::bail!("Не найден iframe с 'История входов'");
}
async fn process_user(driver: &WebDriver,  user_id: u32, full_name: &str, base_url: &str, not_found_list: &mut Vec<String>, 
    mobile_nice: &mut Vec<String>, mobile_no: &mut Vec<String>) -> Result<()>{
        let profile_url = format!("{}/company/personal/user/{}", base_url, user_id);
        println!("LINK::{}", profile_url);
        driver.goto(&profile_url).await?;

        static  mut  FIRST_LOAD: bool = true;
        unsafe {
            if FIRST_LOAD {
                sleep(Duration::from_secs(30)).await; // исправлено
                FIRST_LOAD = false;

            }
        }
        let menu_items = driver.find_all(By::ClassName("menu-item-link-text")).await?;
        if menu_items.len() > 12 {
            menu_items[12].click().await?;
        } else {
            anyhow::bail!("Меню слишком короткое");
        }

        sleep(Duration::from_secs(4)).await;

        let search_input = driver.find(By::Id("INTRANET_USER_LIST_s1_search")).await?;
        search_input.clear().await?;
        search_input.send_keys(full_name).await?;
        search_input.send_keys(Key::Return).await?;
        sleep(Duration::from_secs(2)).await; // важно

        let profile_links = driver.find_all(By::ClassName("user-grid_full-name-label")).await?;
        if profile_links.is_empty() {
            println!("NOT FOUND: {}", full_name);
            not_found_list.push(full_name.to_string());
            append_to_file("NOT_FOUND_NAME.txt", full_name)?;
            return Ok(());
        }

        profile_links[0].click().await?;
        sleep(Duration::from_secs(2)).await;

        click_safety_in_iframe(driver).await?;
        sleep(Duration::from_secs(2)).await;
        click_history_in_iframe(driver).await?;

        let filename = find_table_after_clicking_history(driver, full_name).await?;
        println!("Сохранена история в {}", filename);

        if is_good_android(full_name) {
            mobile_nice.push(full_name.to_string());
            append_to_file("MOBILE_NICE_NAME.txt", full_name)?;
        } else {
            mobile_no.push(full_name.to_string());
            append_to_file("MOBILE_NO.txt", full_name)?;
        }

        Ok(())
}
async fn wait_for_element(driver: &WebDriver, by: By, timeout_secs: u64) -> Result<WebElement> {
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(timeout_secs) {
        if let Ok(el) = driver.find(by.clone()).await {
            return Ok(el);
        }
        sleep(Duration::from_millis(500)).await;
    }
    anyhow::bail!("Элемент не найден за {} секунд", timeout_secs)
}

/// Сохраняет все tbody таблицы (как в Scala)
async fn save_table_tbodies(table: &WebElement, filename: &str) -> Result<()> {
    let tbodies = table.find_all(By::Tag("tbody")).await?;
    println!("Найдено tbody элементов: {}", tbodies.len());

    let mut content = String::new();
    for (idx, tbody) in tbodies.iter().enumerate() {
        let rows = tbody.find_all(By::Tag("tr")).await?;
        println!("   TBODY #{}: {} строк", idx + 1, rows.len());

        // Используем outer_html() вместо attr("outerHTML")
        if let Ok(html) = tbody.outer_html().await {
            content.push_str(&format!("[TBODY #{}]\n", idx + 1));
            content.push_str(&"-".repeat(40));
            content.push('\n');
            content.push_str(&html);
            content.push_str("\n\n");
        } else {
            // Если outer_html не работает, пробуем inner_html
            if let Ok(html) = tbody.inner_html().await {
                content.push_str(&format!("[TBODY #{} (inner)]\n", idx + 1));
                content.push_str(&html);
                content.push_str("\n\n");
            }
        }
    }

    if content.is_empty() {
        println!("⚠️ Внимание: таблица не содержит строк!");
    } else {
        fs::write(filename, &content)?;
        println!("✅ Сохранено {} байт в {}", content.len(), filename);
    }
    Ok(())
}
/// Прямой переход в 4-й iframe (индекс 3) и сохранение таблицы
async fn find_table_in_fourth_iframe_and_save(driver: &WebDriver, user_name: &str) -> Result<String> {
    let filename = format!("{}_history.html", user_name);
    driver.enter_default_frame().await?;

    let iframes = driver.find_all(By::Tag("iframe")).await?;
    println!("Всего iframe на странице: {}", iframes.len());
    if iframes.len() <= 3 {
        anyhow::bail!("Нет iframe с индексом 3, всего iframe: {}", iframes.len());
    }

    driver.enter_frame(3).await?; // 4-й iframe
    println!("✓ Переключились на iframe #4");

    let table = wait_for_element(driver, By::Id("login_history_grid_table"), 10).await?;
    println!("✓ Таблица найдена в iframe #4");
    save_table_tbodies(&table, &filename).await?;

    driver.enter_default_frame().await?;
    Ok(filename)
}

/// Запасной метод: перебор всех iframe
async fn fallback_find_table(driver: &WebDriver, user_name: &str) -> Result<String> {
    let filename = format!("{}_history.html", user_name);
    println!("=== Запасной метод: перебор всех iframe ===");
    driver.enter_default_frame().await?;

    let iframes = driver.find_all(By::Tag("iframe")).await?;
    println!("Найдено iframe для перебора: {}", iframes.len());

    for idx in 0..iframes.len() {
        println!("Проверяем iframe #{}", idx + 1);
        if driver.enter_frame(idx as u16).await.is_ok() {
            if let Ok(table) = driver.find(By::Id("login_history_grid_table")).await {
                println!("✓ Таблица найдена в iframe #{}", idx + 1);
                save_table_tbodies(&table, &filename).await?;
                driver.enter_default_frame().await?;
                return Ok(filename);
            }
            driver.enter_default_frame().await?;
        }
    }
    anyhow::bail!("Таблица не найдена ни в одном iframe")
}

/// Основной метод поиска таблицы (аналог findTableAfterClickingHistory)
async fn find_table_after_clicking_history(driver: &WebDriver, user_name: &str) -> Result<String> {
    println!("=== Поиск таблицы для пользователя: {} ===", user_name);

    // 1. Пробуем найти в текущем контексте (ожидание до 5 секунд)
    if let Ok(table) = wait_for_element(driver, By::Id("login_history_grid_table"), 5).await {
        println!("✓ Таблица найдена в текущем контексте");
        let filename = format!("{}_history.html", user_name);
        save_table_tbodies(&table, &filename).await?;
        return Ok(filename);
    }

    // 2. Пробуем 4-й iframe
    match find_table_in_fourth_iframe_and_save(driver, user_name).await {
        Ok(f) => Ok(f),
        Err(e) => {
            println!("✗ Ошибка при работе с 4-м iframe: {}", e);
            println!("Пробуем альтернативный метод поиска...");
            fallback_find_table(driver, user_name).await
        }
    }
}

// async fn find_and_save_history_table(driver: &WebDriver, user_name: &str) -> Result<String>{
//     let filename = format!("{}_history.html", user_name);
//     if let Ok(table) = driver.find(By::Id("login_history_grid_table")).await{
//         save_table_tbodies(&table, &filename).await?;
//         return Ok(filename);
//     }
//     let iframes = driver.find_all(By::Tag("iframe")).await?;
//     for idx in 0..iframes.len(){
//         if driver.enter_frame(idx as u16).await.is_ok(){
//             if let Ok(table) = driver.find(By::Id("login_history_grid_table")).await{
//                 save_table_tbodies(&table, &filename).await?;
//                 driver.enter_default_frame().await?;
//                 return Ok(filename);
//             }
//             driver.enter_default_frame().await?;
//         }
//     }
//     anyhow::bail!("Таблица истории не найдена ни в одном iframe")
// }

// async fn save_table_tbodies(table: &WebElement, filename: &str) -> Result<()>{
//     let tbodies = table.find_all(By::Tag("tbody")).await?;
//     let mut content = String::new();
//     for tbody in tbodies {
//         let rows = tbody.find_all(By::Tag("tr")).await?;
//         for row in rows {
//             if let Ok(Some(html)) = row.get_attribute("outerHTML").await {
//                 content.push_str(&html);
//                 content.push('\n');
//             }
//         }
//     }
//     fs::write(filename, content)?;
//     println!("Save strings history{}", filename);
//     Ok(())
// }



#[tokio::main]
async fn main() -> Result<()> {
  const USERS_FILE: &str = "USERS";
  const NOT_FOUND_FILE: &str = "NOT_FOUND_NAME.txt";
  const MOBILE_NICE_FILE: &str = "MOBILE_NICE_NAME.txt";
  const MOBILE_NO_FILE: &str = "MOBILE_NO.txt";
  const BASE_URL: &str = "https://relits.bitrix24.ru";

  let users = read_users(USERS_FILE)?;
  if users.is_empty() {
    println!("Нет пользователей в файле");
    return Ok(());
  }

    let caps = DesiredCapabilities::chrome();
  let driver = WebDriver::new("http://localhost:21000", caps).await?; 

  let mut not_found = Vec::new();
  let mut mobile_nice = Vec::new();
  let mut mobile_no = Vec::new();

  for (id, name) in users {
    println!("USER:: {},   index:: {}", name, id);
    if let Err(e) = process_user(&driver, id, &name, BASE_URL, &mut not_found, &mut mobile_nice, &mut mobile_no).await {
      eprintln!("Ошибка при обработке {}: {}", name, e);
    }
  }

  println!("\nNOT FOUND:");
for name in &not_found {
    println!("{}", name);
}
println!("\nGOOD MOBILE:");
for name in &mobile_nice {
    println!("{}", name);
}
println!("\nNO MOBILE:");
for name in &mobile_no {
    println!("{}", name);
}

  driver.quit().await?;
  Ok(())
}