use std::path::Path;

use thirtyfour::prelude::*;
use chrono::prelude::*;
use super::*;

#[derive(strum::FromRepr, Debug, PartialEq)]
#[repr(u8)]
pub enum AppointmentType {
    AbholungAufenthaltserlaubnis,
    RwthStudentenVerlaengerungAufenthaltserlaubnis,
}

#[tauri::command]
pub async fn activate(email: String, appointment_typename: u8) {
    log::trace!("{}",appointment_typename.clone());
    match AppointmentType::from_repr(appointment_typename).unwrap() {
        AppointmentType::AbholungAufenthaltserlaubnis => {
            book_eventloop(email,
                |email| Box::pin(book_abholung_aufenthaltserlaubnis(email)))
                .await;
        },
        AppointmentType::RwthStudentenVerlaengerungAufenthaltserlaubnis => {
            book_eventloop(email,
                |email| Box::pin(book_rwth_studenten_verlaengerung_aufenthaltserlaubnis(email)))
                .await;
        }
    }
}

///
/// 
/// will constantly try to book the appointment in some time interval
pub async fn book_eventloop<F> (email: String, book_fn: F)
where
    F: Fn(String) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<bool>> + Send>>,
{
    loop {
        if let Ok(has_appointment) = book_fn(email.to_owned()).await {
            if has_appointment == true {
                break;
            }
        }

        if (4..9).contains(& Local::now().hour()) {
            tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;
        } else {
            tokio::time::sleep(tokio::time::Duration::from_secs(40)).await;
        }
    }
}

///
/// return Ok<if has availiuble appointment or not>
/// return Err<error occur>
pub async fn book_abholung_aufenthaltserlaubnis(email: String) -> anyhow::Result<bool> {
    let localhost = CONFIG.get("localhost")
        .unwrap()
        .as_str()
        .unwrap();
    let auslaenderamt_url = CONFIG.get("auslaenderamt_url")
        .unwrap()
        .as_str()
        .unwrap();
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless().unwrap();
    caps.set_disable_gpu().unwrap();
    caps.set_no_sandbox().unwrap();
    let driver = WebDriver::new(localhost, caps).await?;
    driver.get(auslaenderamt_url).await?;

    /* step1 */
    dbg!("step1");
    let button = driver
    .find(By::Id("buttonfunktionseinheit-1")) 
    .await?;
    button.click().await?;

    /* step2 */
    dbg!("step2");
    let header = driver.find(By::Id("header_concerns_accordion-454")).await?;
    if let Ok(cookie_accept_button) = driver.find(By::Id("cookie_msg_btn_yes")).await { 
        cookie_accept_button.click().await?;
    }
    header.click().await?;

    // let content = driver
    // .query(By::Id("content_concerns_accordion-454"))
    // .wait(std::time::Duration::from_secs(10), std::time::Duration::from_secs(1))
    // .first()
    // .await?;

    let input_box = driver.find(By::Id("input-299")).await?;
    let is_displayed = input_box.is_displayed().await?;
    log::trace!("{}",is_displayed);
    if !is_displayed {
        driver.execute("arguments[0].scrollIntoView(true);", vec![input_box.to_json()?]).await?;
    }

    input_box.send_keys("1").await?;
    let value = input_box.prop("value").await?.unwrap();
    log::trace!("{}",value);
    
    let weiter_button = driver.find(By::Id("WeiterButton")).await?;
    weiter_button.click().await?;

    driver.execute("document.getElementById('OKButton').click();", vec![]).await?;
    // log::trace!(driver.current_url().await?);

    /* step3 */
    dbg!("step3");
    driver.execute("document.querySelector('input[name=\"select_location\"]').click();", vec![]).await?;
    // log::trace!(driver.current_url().await?);

    /* step4 */
    dbg!("step4");
    let text = driver.find(By::Css("h1"))
        .await?
        .text()
        .await?;
    if text.contains("Keine Zeiten verfügbar") {
        log::info!("appointment not avaliable");
        driver.quit().await?;
        return Ok(false);
    } else if !text.contains("Auswahl der Zeit") {
        log::info!("unexpected situation occur");
        driver.quit().await?;
        return Ok(false);
    }

    let mut dates = Vec::new();// 选其中第一天
    for elem in driver.find_all(By::Css("h3")).await? {
        let text = elem.text().await?;
        dbg!(&text);
        if text.contains("Montag") 
            || text.contains("Dienstag") 
            || text.contains("Mittwoch") 
            || text.contains("Donnerstag") 
            || text.contains("Freitag") 
        {
            dates.push(elem);
        }
    }
    let date = dates[0].clone();
    driver.execute("arguments[0].click();", vec![date.to_json()?]).await?;
    
    let mut times =  Vec::new(); // 选其中第一个时间段
    for elem in driver.find_all(By::Tag("button")).await? {
        if elem.text().await?.contains(":") && elem.is_enabled().await?{
            dbg!(&elem.text().await?);
            times.push(elem);
        }
    }
    driver.execute("arguments[0].click();", vec![times[0].to_json()?]).await?;
    
    for elem in driver.find_all(By::Css("button")).await? {
        if elem.text().await?.contains("Ja") {
            elem.click().await?;
            break;
        }
    }

    /* step5 */
    // 先做图片识别
    let varify_pic =driver.find(By::Id("captcha_image")).await?;
    varify_pic.scroll_into_view().await?;
    varify_pic.screenshot(Path::new("static/verify_pic.png")).await?;

    driver.quit().await?;
    return Ok(true);
}

///
/// return Ok<if has availiuble appointment>
/// return Err<error occur>
pub async fn book_rwth_studenten_verlaengerung_aufenthaltserlaubnis(email: String) -> anyhow::Result<bool> {
    let localhost = CONFIG.get("localhost")
        .unwrap()
        .as_str()
        .unwrap();
    let auslaenderamt_url = CONFIG.get("auslaenderamt_url")
        .unwrap()
        .as_str()
        .unwrap();
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless().unwrap();
    caps.set_disable_gpu().unwrap();
    caps.set_no_sandbox().unwrap();
    let driver = WebDriver::new(localhost, caps).await?;
    driver.get(auslaenderamt_url).await?;

    /* step1 */
    log::trace!("step1");
    let button = driver
    .find(By::Id("buttonfunktionseinheit-1")) 
    .await?;
    button.click().await?;

    /* step2 */
    log::trace!("step2");
    let header = driver.find(By::Id("header_concerns_accordion-455")).await?;
    if let Ok(cookie_accept_button) = driver.find(By::Id("cookie_msg_btn_yes")).await { 
        cookie_accept_button.click().await?;
    }
    header.click().await?;

    let content = driver
    .query(By::Id("content_concerns_accordion-455"))
    .wait(std::time::Duration::from_secs(10), std::time::Duration::from_secs(1))
    .first()
    .await?;

    let input_box = driver.find(By::Id("input-286")).await?;
    let is_displayed = input_box.is_displayed().await?;
    log::trace!("{}",is_displayed);
    if !is_displayed {
        driver.execute("arguments[0].scrollIntoView(true);", vec![input_box.to_json()?]).await?;
    }
    log::trace!("{}",input_box.is_displayed().await?);

    input_box.send_keys("1").await?;
    let value = input_box.prop("value").await?.unwrap();
    dbg!(value);
    
    let weiter_button = driver.find(By::Id("WeiterButton")).await?;
    weiter_button.click().await?;

    driver.execute("document.getElementById('OKButton').click();", vec![]).await?;
    // log::trace!(driver.current_url().await?);

    /* step3 */
    log::trace!("step3");
    driver.execute("document.querySelector('input[name=\"select_location\"]').click();", vec![]).await?;
    // log::trace!(driver.current_url().await?);

    /* step4 */
    log::trace!("step4");
    let text = driver.find(By::Css("h1"))
        .await?
        .text()
        .await?;
    if text.contains("Auswahl der Zeit") {
        log::info!("appointment avaliable");
        let _ = notify(email,"book_rwth_studenten_verlaengerung_aufenthaltserlaubnis".to_string());
        driver.quit().await?;
        return Ok(true);
    } else if text.contains("Keine Zeiten verfügbar") {
        log::info!("appointment not avaliable");
        driver.quit().await?;
        return Ok(false);
    } else {
        log::info!("unexpected situation occur");
        driver.quit().await?;
        return Ok(false);
    }
}


#[tokio::test]
async fn test() {
    let caps = DesiredCapabilities::firefox();
    let driver = WebDriver::new("http://localhost:4444", caps).await.unwrap();
}

#[test]
fn test_time() {
    let formatted_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    log::trace!("{}",formatted_time);
    // std::thread::sleep(std::time::Duration::from_secs(10));
    log::trace!("{}",Local::now());
}

#[tokio::test]
async fn test_pass_fn() {
    let email = "example.com".to_string();
    book_eventloop(email, |email| Box::pin(book_abholung_aufenthaltserlaubnis(email.to_owned()))).await;
}

#[tokio::test]
async fn test_book_abholung() {
    let _ = book_abholung_aufenthaltserlaubnis("zhouyi.guan@rwth-aachen.de".to_string()).await;
}