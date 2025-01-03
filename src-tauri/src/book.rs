use super::*;
use chrono::prelude::*;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, EventTarget};
use thirtyfour::prelude::*;

pub mod ocr;
pub use ocr::*;

pub mod asr;
pub use asr::*;

pub mod interact;
pub use interact::*;

#[derive(strum::FromRepr, Debug, PartialEq, Clone)]
#[repr(u8)]
pub enum AppointmentType {
    AbholungAufenthaltserlaubnis,
    RwthStudentenVerlaengerungAufenthaltserlaubnis,
    NotDefined,
}

///
///
/// 启动后开始不间断访问网站获取termin
#[tauri::command]
pub async fn book(email: String, appointment_typename: u8) {
    let appointment_type = AppointmentType::from_repr(appointment_typename).unwrap();

    /* 注册command到app中,并获取到此次command的id */
    let command_status = CommandStatus::Book {
        user_info: UserInfo {
            vorname: "zhouyi".to_string(),
            nachname: "guan".to_string(),
            email: email,
            telefonnummer: "015226221699".to_string(),
            geburtsdatum: [26, 07, 1999],
        },
        appointment_type: appointment_type.to_owned(),
    };
    let uid = get_app_ins().unwrap().add_command(command_status);

    /* 处理command */
    match appointment_type {
        AppointmentType::AbholungAufenthaltserlaubnis => {
            book_eventloop(uid, |uid| Box::pin(book_abholung_aufenthaltserlaubnis(uid))).await;
        }
        AppointmentType::RwthStudentenVerlaengerungAufenthaltserlaubnis => {
            book_eventloop(uid, |uid| {
                Box::pin(book_rwth_studenten_verlaengerung_aufenthaltserlaubnis(uid))
            })
            .await;
        }
        AppointmentType::NotDefined => {}
    }
}

///
///
/// will constantly try to book the appointment in some time interval
pub async fn book_eventloop<F>(uid: uuid::Uuid, book_fn: F)
where
    F: Fn(
        uuid::Uuid,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<bool>> + Send>>,
{
    loop {
        dbg!("start new book attempt");
        match book_fn(uid).await {
            Ok(book_success) => {
                dbg!(book_success);
                if book_success {
                    break;
                }
            }
            Err(e) => {
                dbg!(e);
            }
        }

        dbg!("no appointment, will retry");
        if (4..9).contains(&Local::now().hour()) {
            tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;
        } else {
            tokio::time::sleep(tokio::time::Duration::from_secs(40)).await;
        }
    }
}

///
/// return Ok<if has availiuble appointment or not>
/// return Err<error occur>
pub async fn book_abholung_aufenthaltserlaubnis(uid: uuid::Uuid) -> anyhow::Result<bool> {
    let app = get_app_ins()?;
    let localhost = app.get_config_value("localhost");
    let auslaenderamt_url = app.get_config_value("auslaenderamt_url");
    let download_dir = std::env::current_dir()
        .unwrap()
        .join("static")
        .join("download")
        .join(uid.to_string());
    std::fs::create_dir_all(&download_dir)?;

    /*
        chromedriver 设置;设置一些浏览器基础交互行为
    */
    let mut caps = DesiredCapabilities::chrome();
    caps.insert_browser_option(
        "prefs",
        serde_json::json!({
            "download.default_directory": download_dir.to_owned().to_str().unwrap(),
            "download.prompt_for_download": false,
            "safebrowsing.enabled": true,
        }),
    )?;
    caps.set_headless().unwrap();
    caps.set_disable_gpu().unwrap();
    caps.set_no_sandbox().unwrap();
    let driver = WebDriver::new(localhost, caps).await?;
    driver.get(auslaenderamt_url).await?;

    /*
        step1
    */
    dbg!("step1");
    let button = driver.find(By::Id("buttonfunktionseinheit-1")).await?;
    dbg!(button.check(&driver).await?.click().await?);

    /*
        step2
    */
    dbg!("step2");
    let header = driver.find(By::Id("header_concerns_accordion-454")).await?;
    dbg!(header.check(&driver).await?.click().await?);

    let input_box = driver.find(By::Id("input-299")).await?;

    dbg!(input_box.check(&driver).await?.send_keys("1").await?);

    let weiter_button = driver.find(By::Id("WeiterButton")).await?;
    dbg!(weiter_button.check(&driver).await?.click().await?);

    driver
        .execute("document.getElementById('OKButton').click();", vec![])
        .await?;

    /*
        step3
    */
    dbg!("step3");
    driver
        .execute(
            "document.querySelector('input[name=\"select_location\"]').click();",
            vec![],
        )
        .await?;

    /*
        step4
    */
    dbg!("step4");
    let text = driver.find(By::Css("h1")).await?.text().await?;
    if text.contains("Keine Zeiten verfügbar") {
        log::info!("appointment not avaliable");
        driver.quit().await?;
        return Ok(false);
    } else if !text.contains("Auswahl der Zeit") {
        log::info!("unexpected situation occur");
        driver.quit().await?;
        return Ok(false);
    }

    /* TODO 在这里可以先提醒*/
    let mut dates = Vec::new(); // 选其中第一天
    for elem in driver.find_all(By::Css("h3")).await? {
        let text = elem.text().await?;
        if text.contains("Montag")
            || text.contains("Dienstag")
            || text.contains("Mittwoch")
            || text.contains("Donnerstag")
            || text.contains("Freitag")
        {
            dates.push(elem);
        }
    }
    dbg!(dates[0].text().await?);
    driver
        .execute("arguments[0].click();", vec![dates[0].to_json()?])
        .await?;

    let mut times = Vec::new(); // 选其中第一个时间段
    for elem in driver.find_all(By::Tag("button")).await? {
        if elem.text().await?.contains(":") && elem.is_enabled().await? {
            times.push(elem);
        }
    }

    dbg!(times[0].text().await?);
    driver
        .execute("arguments[0].click();", vec![times[0].to_json()?])
        .await?;

    for elem in driver.find_all(By::Css("button")).await? {
        if elem.text().await?.contains("Ja") {
            elem.click().await?;
            break;
        }
    }

    /*
        step5
        输入个人信息
        验证码识别 图片/语音
    */
    dbg!("step5");
    let user_info = match app.get_command_status(uid)? {
        CommandStatus::Book { user_info, .. } => user_info,
        CommandStatus::UndefinedCommand { .. } => {
            panic!();
        }
    };
    dbg!(&user_info);

    let vorname_box = driver.find(By::Id("vorname")).await?;
    vorname_box
        .check(&driver)
        .await?
        .send_keys(user_info.vorname)
        .await?;
    let nachname_box = driver.find(By::Id("nachname")).await?;
    nachname_box
        .check(&driver)
        .await?
        .send_keys(user_info.nachname)
        .await?;
    let email_box = driver.find(By::Id("email")).await?;
    email_box
        .check(&driver)
        .await?
        .send_keys(user_info.email.clone())
        .await?;
    let emailwhlg_box = driver.find(By::Id("emailwhlg")).await?;
    emailwhlg_box
        .check(&driver)
        .await?
        .send_keys(user_info.email)
        .await?;
    let tel_box = driver.find(By::Id("tel")).await?;
    tel_box
        .check(&driver)
        .await?
        .send_keys(user_info.telefonnummer)
        .await?;
    let geburtsdatum_day_box = driver.find(By::Id("geburtsdatumDay")).await?;
    geburtsdatum_day_box
        .check(&driver)
        .await?
        .send_keys(user_info.geburtsdatum[0].to_string())
        .await?;
    let geburtsdatum_month_box = driver.find(By::Id("geburtsdatumMonth")).await?;
    geburtsdatum_month_box
        .check(&driver)
        .await?
        .send_keys(user_info.geburtsdatum[1].to_string())
        .await?;
    let geburtsdatum_year_box = driver.find(By::Id("geburtsdatumYear")).await?;
    geburtsdatum_year_box
        .check(&driver)
        .await?
        .send_keys(user_info.geburtsdatum[2].to_string())
        .await?;

    let checkbox = driver.find(By::Id("privacy_check_agreement")).await?;
    checkbox.check(&driver).await?.click().await?;
    // checkbox.screenshot(&download_dir.join("chcekbox.png")).await?;

    /* 图片识别 */
    let captcha_image_path = download_dir.join("captcha_image.png");
    let captcha_image = driver.find(By::Id("captcha_image")).await?;
    captcha_image
        .check(&driver)
        .await?
        .screenshot(&captcha_image_path)
        .await?;
    let captcha_image_str = ocr(captcha_image_path.to_str().unwrap())?;
    dbg!(captcha_image_str);
    /* 语音识别 这里是需要下载音频文件; 必须使用当前模拟的浏览器driver发送http请求下载 不能用reqwest之类的发送http请求 因为是https协议 用第三方发送请求得到的回应是不匹配的 */
    let captcha_audio = driver.find(By::Id("captcha_image_source_wav")).await?;
    let captcha_audio_url_rel = dbg!(captcha_audio.attr("src").await?.unwrap());
    let mut captcha_audio_url = auslaenderamt_url.to_string();
    captcha_audio_url.push('/');
    captcha_audio_url.push_str(&captcha_audio_url_rel);
    driver
        .execute(
            &format!("window.location.href = '{}';", captcha_audio_url),
            vec![],
        )
        .await?;
    /* chromedriver不会等待下载完成 所以这里需要主动等待 */
    let captcha_audio_path: PathBuf;
    loop {
        if let Some(path) = has_wav_files(&download_dir) {
            captcha_audio_path = path;
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    let captcha_audio_str = asr(captcha_audio_path.to_str().unwrap())?;
    dbg!(&captcha_audio_str);

    /* 输入验证结果 */
    let captcha_result_box = driver.find(By::Id("captcha_result")).await?;
    captcha_result_box
        .check(&driver)
        .await?
        .send_keys(captcha_audio_str)
        .await?;
    /* 刷新 */
    // let reload_button = driver.find(By::Id("captcha_reload")).await?;
    // reload_button.check(&driver).await?.click().await?;
    driver.screenshot(&download_dir.join("step5.png")).await?;

    /* 提交 */
    let submit_button = driver.find(By::Id("chooseTerminButton")).await?;
    submit_button.check(&driver).await?.click().await?;

    /*
        step6
    */
    driver.screenshot(&download_dir.join("step6.png")).await?;
    if let Ok(_error_element) = driver.find(By::Css("div.content__error")).await {
        dbg!("wrong input userdata in step5");
        std::fs::remove_dir_all(&download_dir)?;
        driver.quit().await?;
        return Ok(false);
    }

    /*
        结束
    */
    std::fs::remove_dir_all(&download_dir)?;
    driver.quit().await?;
    return Ok(true);
}

///
/// return Ok<if has availiuble appointment>
/// return Err<error occur>
pub async fn book_rwth_studenten_verlaengerung_aufenthaltserlaubnis(
    uid: uuid::Uuid,
) -> anyhow::Result<bool> {
    /* let localhost = CONFIG.get("localhost").unwrap().as_str().unwrap();
    let auslaenderamt_url = CONFIG.get("auslaenderamt_url").unwrap().as_str().unwrap();
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless().unwrap();
    caps.set_disable_gpu().unwrap();
    caps.set_no_sandbox().unwrap();
    let driver = WebDriver::new(localhost, caps).await?;
    driver.get(auslaenderamt_url).await?;

    /* step1 */
    log::trace!("step1");
    let button = driver.find(By::Id("buttonfunktionseinheit-1")).await?;
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
        .wait(
            std::time::Duration::from_secs(10),
            std::time::Duration::from_secs(1),
        )
        .first()
        .await?;

    let input_box = driver.find(By::Id("input-286")).await?;
    let is_displayed = input_box.is_displayed().await?;
    log::trace!("{}", is_displayed);
    if !is_displayed {
        driver
            .execute(
                "arguments[0].scrollIntoView(true);",
                vec![input_box.to_json()?],
            )
            .await?;
    }
    log::trace!("{}", input_box.is_displayed().await?);

    input_box.send_keys("1").await?;
    let value = input_box.prop("value").await?.unwrap();
    dbg!(value);

    let weiter_button = driver.find(By::Id("WeiterButton")).await?;
    weiter_button.click().await?;

    driver
        .execute("document.getElementById('OKButton').click();", vec![])
        .await?;
    // log::trace!(driver.current_url().await?);

    /* step3 */
    log::trace!("step3");
    driver
        .execute(
            "document.querySelector('input[name=\"select_location\"]').click();",
            vec![],
        )
        .await?;
    // log::trace!(driver.current_url().await?);

    /* step4 */
    log::trace!("step4");
    let text = driver.find(By::Css("h1")).await?.text().await?;
    if text.contains("Auswahl der Zeit") {
        log::info!("appointment avaliable");
        let _ = notify(
            email,
            "book_rwth_studenten_verlaengerung_aufenthaltserlaubnis".to_string(),
        );
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
    } */

    Ok(true)
}

#[tokio::test]
async fn test_book_abholung() {
    // if let Err(e) =
    //     book_abholung_aufenthaltserlaubnis("zhouyi.guan@rwth-aachen.de".to_string()).await
    // {
    //     dbg!(e);
    // }
}

#[test]
fn test_localdir() {
    let dir = dbg!(std::env::current_dir()
        .unwrap()
        .join("static")
        .join("download"));
    let dir = dir.to_str().unwrap();

    std::fs::create_dir_all(dir).unwrap();
}

#[tokio::test]
async fn test_download_audio() {
    use std::io::Write;
    const URL: &str = "https://termine.staedteregion-aachen.de/auslaenderamt/app/securimage/securimage_play.php?id=465c8cde7e745833b0e842e372027cc4";

    let client = reqwest::Client::new();
    let response = client.get(URL).send().await.unwrap();
    if response.status().is_success() {
        let mut file = std::fs::File::create("static/asr/test.wav").unwrap();
        let content = response.bytes().await.unwrap();
        file.write_all(&content).unwrap();
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
    log::trace!("{}", formatted_time);
    // std::thread::sleep(std::time::Duration::from_secs(10));
    log::trace!("{}", Local::now());
}

#[tokio::test]
async fn test_pass_fn() {
    // let email = "example.com".to_string();
    // book_eventloop(email, |email| {
    //     Box::pin(book_abholung_aufenthaltserlaubnis(email.to_owned()))
    // })
    // .await;
}
