use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::print;
use std::sync::Arc;
use thirtyfour::By;
use thirtyfour::Key;
use thirtyfour::prelude::*;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
use std::time::Instant;


pub const PASS_FIELNAME: &str = "pass";
const LOGIN_FILENAME: &str = "login";
const CLEANUP_ON: bool = true;
const URL_WS_CONNECT: &str = "ws://127.0.0.1:3000/proc";


fn clean_up() -> std::io::Result<()> {
    let entries = fs::read_dir(".")?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "html" {
                    fs::remove_file(&path)?;
                }
            }
        }
    }
    Ok(())
}


fn pass() -> Option<String> {
    read_from_file(PASS_FIELNAME)
}

fn login() -> Option<String> {
    read_from_file(LOGIN_FILENAME)
}

fn read_from_file(filename: &str) -> Option<String> {
    let g = fs::read_to_string(filename);
    match g {
        Ok(str) => Some(str),
        Err(_) => None,
    }
}

fn read_users(file_path: &str) -> Result<Vec<(u32, String)>> {
    let file = File::open(file_path).context("unable to open file USERS")?;
    let reader = BufReader::new(file);
    let mut users = Vec::new();

    for line in reader.lines() {
        let line__ = line?;
        if line__.trim().is_empty() {
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

fn append_to_file(fila_name: &str, data: &str) -> Result<()> {
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
    if let Ok(html) = fs::read_to_string(&filename) {
        html.contains("iOS") || html.contains("Android")
    } else {
        false
    }
}

fn is_no_desktop(user_name: &str) -> bool {

    println!("NAME:: {}", user_name);
    let filename = format!("{}_history.html", user_name);
    if let Ok(html) = fs::read_to_string(&filename){
        let res = !html.contains("Windows")  &&   !html.contains("Win");
        if res { print!("!!! its no desktop there!!!!")} 
        else {
                println!("DESKTOP MPRESENT!");
        }
        res
    } else {
        false
    }
}
// async fn click_safety_in_iframe(driver: &WebDriver) -> Result<()> {
//     sleep(Duration::from_secs(2)).await;
//     let iframes = driver.find_all(By::Tag("iframe")).await?;
//     for (idx, iframe) in iframes.iter().enumerate() {
//         if let Ok(_) = driver.enter_frame(idx as u16).await {
//             let elements = driver.find_all(By::ClassName("ui-btn-text-inner")).await?;
//             for el in elements {
//                 let text = el.text().await.unwrap_or_default();
//                 if text.contains("Безопасность") {
//                     el.click().await?;
//                     driver.enter_default_frame().await?;
//                     return Ok(());
//                 }
//             }
//             driver.enter_default_frame().await?;
//         }
//     }
//     anyhow::bail!("Не найден iframe с кнопкой 'Безопасность'");
// }


async fn click_safety_in_iframe(driver: &WebDriver) -> Result<()> {
    sleep(Duration::from_secs(3)).await;
    let iframes = driver.find_all(By::Tag("iframe")).await?;
    for (idx, _) in iframes.iter().enumerate() {
        if driver.enter_frame(idx as u16).await.is_ok() {
            // Ищем кнопку с текстом "Безопасность" (можно искать по XPath)
            let btn = driver
                .query(By::XPath("//*[contains(text(), 'Безопасность')]"))
                .wait(Duration::from_secs(5), Duration::from_millis(500))
                .and_clickable()
                .first()
                .await;
        //    // if let Ok(b) = btn {
        //    //     b.scroll_into_view().await?;
        //    //     // Пробуем кликнуть, если не выходит — через JS
        //    //     if let Err(e) = b.click().await {
        //    //         driver.execute("arguments[0].click();", vec![b.to_json()?]).await?;
        //    //     }
        //    //     driver.enter_default_frame().await?;
        //    //     return Ok(());
        //    // }
            if let Ok(b) = btn {
                b.scroll_into_view().await?;
                driver.execute("arguments[0].click();", vec![b.to_json()?]).await?;
                driver.enter_default_frame().await?;
                return Ok(());
            }
            driver.enter_default_frame().await?;
        }
    }
    anyhow::bail!("Кнопка 'Безопасность' не найдена ни в одном iframe");
}






async fn click_history_in_iframe(driver: &WebDriver) -> Result<()> {
    sleep(Duration::from_secs(2)).await;
    let iframes = driver.find_all(By::Tag("iframe")).await?;
    for (idx, _) in iframes.iter().enumerate() {
        if driver.enter_frame(idx as u16).await.is_ok() {
            let btn = driver
                .query(By::XPath("//*[contains(text(), 'История входов')]"))
                .wait(Duration::from_secs(5), Duration::from_millis(500))
                .and_clickable()
                .first()
                .await;
//            // if let Ok(b) = btn {
//            //     b.scroll_into_view().await?;
//            //     if let Err(e) = b.click().await {
//            //         driver.execute("arguments[0].click();", vec![b.to_json()?]).await?;
//            //     }
//            //     driver.enter_default_frame().await?;
//            //     return Ok(());
//            // }
            if let Ok(b) = btn {
                b.scroll_into_view().await?;
                driver.execute("arguments[0].click();", vec![b.to_json()?]).await?;
                driver.enter_default_frame().await?;
                return Ok(());
            }
            driver.enter_default_frame().await?;
        }
    }
    anyhow::bail!("Кнопка 'История входов' не найдена");
}



// async fn click_history_in_iframe(driver: &WebDriver) -> Result<()> {
//     sleep(Duration::from_secs(2)).await;
//     let iframes = driver.find_all(By::Tag("iframe")).await?;
//     for (idx, iframe) in iframes.iter().enumerate() {
//         if let Ok(_) = driver.enter_frame(idx as u16).await {
//             let elements = driver
//                 .find_all(By::XPath(
//                     "//div[@class='ui-sidepanel-menu-link-text' and text()='История входов']",
//                 ))
//                 .await?;
//             if !elements.is_empty() {
//                 elements[0].click().await?;
//                 driver.enter_default_frame().await?;
//                 return Ok(());
//             }
//             driver.enter_default_frame().await?;
//         }
//     }
//     anyhow::bail!("Не найден iframe с 'История входов'");
// }
async fn process_user(
    driver: &WebDriver,
    user_id: u32,
    full_name: &str,
    base_url: &str,
    not_found_list: &mut Vec<String>,
    mobile_nice: &mut Vec<String>,
    mobile_no: &mut Vec<String>,
    desktop_no: &mut Vec<String>
) -> Result<()> {
    let profile_url = format!("{}/company/personal/user/{}", base_url, user_id);
    println!("LINK::{}", profile_url);
    driver.goto(&profile_url).await?;

    static mut FIRST_LOAD: bool = true;
    unsafe {
        if FIRST_LOAD {
            sleep(Duration::from_secs(30)).await;
            FIRST_LOAD = false;
        }
    }
    // let menu_items = driver
    //     .find_all(By::ClassName("menu-item-link-text"))
    //     .await?;
    // if menu_items.len() > 12 {
    //     menu_items[12].click().await?;
    // } else {
    //     anyhow::bail!("Меню слишком короткое");
    // }


    let menu_items = driver.find_all(By::ClassName("menu-item-link-text")).await?;
    if menu_items.len() > 12 {
        let target = &menu_items[12];
        target.scroll_into_view().await?;
        driver.execute("arguments[0].click();", vec![target.to_json()?]).await?;
    } else {
        anyhow::bail!("Меню слишком короткое");
    }

    sleep(Duration::from_secs(4)).await;

    let search_input = driver.find(By::Id("INTRANET_USER_LIST_s1_search")).await?;
    search_input.clear().await?;
    search_input.send_keys(full_name).await?;
    search_input.send_keys(Key::Return).await?;
    sleep(Duration::from_secs(2)).await;

//    // let profile_links = driver
//    //     .find_all(By::ClassName("user-grid_full-name-label"))
//    //     .await?;
    let profile_links = driver
    .query(By::ClassName("user-grid_full-name-label"))
    .wait(Duration::from_secs(10), Duration::from_millis(500))
    .all()
    .await?;




    if profile_links.is_empty() {
        println!("NOT FOUND: {}", full_name);
        not_found_list.push(full_name.to_string());
    //    append_to_file("NOT_FOUND_NAME.txt", full_name)?;
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
    //    append_to_file("MOBILE_NICE_NAME.txt", full_name)?;
    } else {
        mobile_no.push(full_name.to_string());
    //    append_to_file("MOBILE_NO.txt", full_name)?;
    }

    if is_no_desktop(full_name) {
        println!("ADDED  no desktop{}", full_name);
        desktop_no.push(full_name.to_string());
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

async fn save_table_tbodies(table: &WebElement, filename: &str) -> Result<()> {
    let tbodies = table.find_all(By::Tag("tbody")).await?;
    println!("Найдено tbody элементов: {}", tbodies.len());

    let mut content = String::new();
    for (idx, tbody) in tbodies.iter().enumerate() {
        let rows = tbody.find_all(By::Tag("tr")).await?;
        println!("   TBODY #{}: {} строк", idx + 1, rows.len());

        if let Ok(html) = tbody.outer_html().await {
            content.push_str(&format!("[TBODY #{}]\n", idx + 1));
            content.push_str(&"-".repeat(40));
            content.push('\n');
            content.push_str(&html);
            content.push_str("\n\n");
        } else {
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
async fn find_table_in_fourth_iframe_and_save(
    driver: &WebDriver,
    user_name: &str,
) -> Result<String> {
    let filename = format!("{}_history.html", user_name);
    driver.enter_default_frame().await?;

    let iframes = driver.find_all(By::Tag("iframe")).await?;
    println!("Всего iframe на странице: {}", iframes.len());
    if iframes.len() <= 3 {
        anyhow::bail!("Нет iframe с индексом 3, всего iframe: {}", iframes.len());
    }

    driver.enter_frame(3).await?;
    println!("✓ Переключились на iframe #4");

    let table = wait_for_element(driver, By::Id("login_history_grid_table"), 10).await?;
    println!("✓ Таблица найдена в iframe #4");
    save_table_tbodies(&table, &filename).await?;

    driver.enter_default_frame().await?;
    Ok(filename)
}

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

async fn find_table_after_clicking_history(driver: &WebDriver, user_name: &str) -> Result<String> {
    println!("=== Поиск таблицы для пользователя: {} ===", user_name);
    if let Ok(table) = wait_for_element(driver, By::Id("login_history_grid_table"), 5).await {
        println!("✓ Таблица найдена в текущем контексте");
        let filename = format!("{}_history.html", user_name);
        save_table_tbodies(&table, &filename).await?;
        return Ok(filename);
    }

    match find_table_in_fourth_iframe_and_save(driver, user_name).await {
        Ok(f) => Ok(f),
        Err(e) => {
            println!("✗ Ошибка при работе с 4-м iframe: {}", e);
            println!("Пробуем альтернативный метод поиска...");
            fallback_find_table(driver, user_name).await
        }
    }
}

async fn login_cad(driver: &WebDriver, username: &str, pass: &str) -> Result<()> {
    // Поле логина
    let login_field = driver
        .query(By::Css("input.b24net-text-input__field[type='text']"))
        .wait(Duration::from_secs(10), Duration::from_millis(500))
        .and_clickable()
        .first()
        .await
        .context("Поле логина не появилось")?;
    login_field.send_keys(username).await?;

    // Кнопка "Продолжить" (первая)
    let continue_btn = driver
        .query(By::Css(".b24net-login-enter-form__continue-btn"))
        .wait(Duration::from_secs(10), Duration::from_millis(500))
        .and_clickable()
        .first()
        .await
        .context("Кнопка 'Продолжить' не появилась")?;
    continue_btn.click().await?;

    // Поле пароля
    let password_field = driver
        .query(By::Css("input.b24net-text-input__field[type='password']"))
        .wait(Duration::from_secs(30), Duration::from_millis(500))
        .and_clickable()
        .first()
        .await
        .context("Поле пароля не появилось")?;
    password_field.send_keys(pass).await?;

    // Кнопка "Продолжить" (вторая)
    let submit_btn = driver
        .query(By::Css(".b24net-password-enter-form__continue-btn"))
        .wait(Duration::from_secs(10), Duration::from_millis(500))
        .and_clickable()
        .first()
        .await
        .context("Кнопка 'Продолжить' после пароля не появилась")?;
    submit_btn.click().await?;

    Ok(())
}

async fn spawn_cdriver_and_login(base_url: String) -> WebDriver {
    let mut caps = DesiredCapabilities::chrome();
    let _ = caps.add_arg("--no-sandbox");
    let _ = caps.add_arg("--disable-dev-shm-usage");
    let _ = caps.add_arg("--window-size=1920,1080");

    let driver: WebDriver = WebDriver::new("http://localhost:21000", caps)
        .await
        .expect("shit happens");
    let user_id = 1;
    let profile_url = format!("{}/company/personal/user/{}", base_url, user_id);
    let _ = driver.goto(&profile_url).await;
    let _ = login_cad(&driver, login().unwrap().as_str(), pass().unwrap().as_str()).await;
    driver
}

// fn split_vec_4_vec(input: Vec<(u32, String)>, split_to: u32) -> Vec<Vec<(u32, String)>> {
//     let mut res: Vec<Vec<(u32, String)>> = vec![];
//     let mut cur_vec: Vec<(u32, String)> = vec![];

//     let mut cur_idx = 1;
//     for (id, name) in input {
//         println!("USER:: {}, index:: {}", name, id);
//         if cur_idx == split_to {
//             res.push(cur_vec);
//             cur_idx = 1;
//             cur_vec = vec![];
//         }
//         cur_vec.push((id, name));
//         cur_idx += 1;
//     }
//     res
// }

fn split_vec_4_vec(input: Vec<(u32, String)>, split_to: u32) -> Vec<Vec<(u32, String)>> {
    let mut res: Vec<Vec<(u32, String)>> = vec![Vec::new(); split_to as usize];
    for (idx, item) in input.into_iter().enumerate() {
        let group_idx = idx % (split_to as usize);
        res[group_idx].push(item);
    }
    res
}


#[tokio::main]
async fn main() -> Result<()> {
    let start = Instant::now();
    let _ =  clean_up();
    println!("Начало выполнения: {:?}", start);
    const USERS_FILE: &str = "USERS";
    const NOT_FOUND_FILE: &str = "NOT_FOUND_NAME.txt";
    const MOBILE_NICE_FILE: &str = "MOBILE_NICE_NAME.txt";
    const MOBILE_NO_FILE: &str = "MOBILE_NO.txt";
    const DESKTOP_NO_FILE: &str = "DESKTOP_NO.txt";

    let users = read_users(USERS_FILE)?;
    if users.is_empty() {
        println!("Нет пользователей в файле");
        return Ok(());
    }

    let mut caps = DesiredCapabilities::chrome();
    let _ = caps.add_arg("--no-sandbox");
    let _ = caps.add_arg("--disable-dev-shm-usage");
    let mut driver_pack: Vec<WebDriver> = vec![];

    let number_drivers: u32 = 8;//8;

    let BASE_URL: String = "https://relits.bitrix24.ru".to_string();
    let BASE_URL2: String = "https://relits.bitrix24.ru".to_string();
    let BASE_URL3 = "https://relits.bitrix24.ru";

    

 {  // multythreadf

    let not_found       = Arc::new(Mutex::new(Vec::new()));
    let mobile_nice     = Arc::new(Mutex::new(Vec::new()));
    let mobile_no       = Arc::new(Mutex::new(Vec::new()));
    let desktop_no: Arc<Mutex<Vec<String>>>      = Arc::new(Mutex::new(Vec::new()));

    let mut id_vecs_4_druver: Vec<Vec<(u32, String)>> = vec![];

   // let splitted_id_users = split_vec_4_vec(users, number_drivers);

   let groups = split_vec_4_vec(users, number_drivers);
    for i in 1..=number_drivers {
        let driver = spawn_cdriver_and_login(BASE_URL.clone()).await;
        driver_pack.push(driver);

       // sleep(Duration::from_secs(2)).await;

    }


    let mut tasks = Vec::new();
    for (i, group) in groups.into_iter().enumerate() {
        let driver = driver_pack[i].clone();
        let not_found_clone = not_found.clone();
        let mobile_nice_clone = mobile_nice.clone();
        let mobile_no_clone = mobile_no.clone();
        let desktop_no_clone = desktop_no.clone();

        let task = tokio::spawn(async move {
            // Локальные буферы для этой задачи
            let mut local_not_found = Vec::new();
            let mut local_mobile_nice = Vec::new();
            let mut local_mobile_no = Vec::new();
            let mut local_desktop_no = Vec::new();

            for (id, name) in group {
                println!("Поток {} обрабатывает {} (ID {})", i, name, id);
                if let Err(e) = process_user(
                    &driver,
                    id,
                    &name,
                    BASE_URL3,
                    &mut local_not_found,
                    &mut local_mobile_nice,
                    &mut local_mobile_no,
                    &mut local_desktop_no
                )
                .await
                {
                    eprintln!("Ошибка при обработке {}: {}", name, e);
                }
            }

            // Добавляем локальные результаты в глобальные векторы
            {
                let mut guard = not_found_clone.lock().await;
                guard.extend(local_not_found);
            }
            {
                let mut guard = mobile_nice_clone.lock().await;
                guard.extend(local_mobile_nice);
            }
            {
                let mut guard = mobile_no_clone.lock().await;
                guard.extend(local_mobile_no);
            }


            {
                let mut guard = desktop_no_clone.lock().await;
                guard.extend(local_desktop_no);
            }
            Ok::<_, anyhow::Error>(())
        });
        tasks.push(task);
    }

    // Ожидаем завершения всех задач
    for task in tasks {
        if let Err(e) = task.await {
            eprintln!("Задача завершилась с ошибкой: {:?}", e);
        }
    }

    let not_found_guard = not_found.lock().await;
    println!("\nNOT FOUND:");
    for name in not_found_guard.iter() {
        println!("{}", name);
    }
    let mobile_nice_guard = mobile_nice.lock().await;
    println!("\nGOOD MOBILE:");
    for name in mobile_nice_guard.iter() {
        println!("{}", name);
    }

    let mobile_no_guard = mobile_no.lock().await;
    println!("\nNO MOBILE:");
    for name in mobile_no_guard.iter() {
        println!("{}", name);
    }


    let no_desktop_guard = desktop_no.lock().await;
    println!("\nNO DESKTOP:");
    for name in no_desktop_guard.iter() {
        println!("{}", name);
    }
    for dr in driver_pack{
        dr.quit().await?;
    }


    std::fs::write(NOT_FOUND_FILE, not_found_guard.join("\n"))?;
    std::fs::write(MOBILE_NICE_FILE, mobile_nice_guard.join("\n"))?;
    std::fs::write(MOBILE_NO_FILE, mobile_no_guard.join("\n"))?;
    std::fs::write(DESKTOP_NO_FILE, no_desktop_guard.join("\n"))?;





 }

  




  {  //one thread

    // let mut not_found1 = vec![];
    // let mut mobile_nice1= vec![];
    // let mut mobile_no1 = vec![];

    // let driver: WebDriver = spawn_cdriver_and_login(BASE_URL2).await;



    //   for (id, name) in users {
    //     println!("USER:: {},   index:: {}", name, id);
    //     if let Err(e) = process_user(&driver, id, &name, BASE_URL3, &mut not_found1, &mut mobile_nice1, &mut mobile_no1).await {
    //       eprintln!("Ошибка при обработке {}: {}", name, e);
    //     }
    //   }

    // println!("\nNOT FOUND:");
    // for name in &not_found1 {
    //     println!("{}", name);
    // }
    // println!("\nGOOD MOBILE:");
    // for name in &mobile_nice1 {
    //     println!("{}", name);
    // }
    // println!("\nNO MOBILE:");
    // for name in &mobile_no1 {
    //     println!("{}", name);
    // }

    // driver.quit().await?;

  }

    let duration = start.elapsed();

    if CLEANUP_ON{
        let _ = clean_up();
    }

    println!("Завершение. Выполнение заняло: {:?}", duration);

    Ok(())
}
