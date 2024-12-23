use anyhow::{anyhow,Result};
use thirtyfour::prelude::*;
use super::*;

pub async fn book_abholung_aufenthaltserlaubnis(email: &String) -> Result<()> {
    let localhost = CONFIG.get("localhost")
        .unwrap()
        .as_str()
        .unwrap();
    let auslaenderamt_url = CONFIG.get("auslaenderamt_url")
        .unwrap()
        .as_str()
        .unwrap();
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new(localhost, caps).await?;
    driver.get(auslaenderamt_url).await?;

    /* step1 */
    let button = driver
    .find(By::Id("buttonfunktionseinheit-1")) 
    .await?;
    button.click().await?;

    /* step2 */
    dbg!(driver.current_url().await?);

    let header = driver.find(By::Id("header_concerns_accordion-454")).await?;
    if let Ok(cookie_accept_button) = driver.find(By::Id("cookie_msg_btn_yes")).await { 
        cookie_accept_button.click().await?;
    }
    header.click().await?;

    let content = driver
    .query(By::Id("content_concerns_accordion-454"))
    .wait(std::time::Duration::from_secs(10), std::time::Duration::from_secs(1))
    .first()
    .await?;

    let input_box = driver.find(By::Id("input-299")).await?;
    let is_displayed = input_box.is_displayed().await?;
    dbg!(is_displayed);
    if !is_displayed {
        driver.execute("arguments[0].scrollIntoView(true);", vec![input_box.to_json()?]).await?;
    }

    input_box.send_keys("1").await?;
    let value = input_box.prop("value").await?.unwrap();
    dbg!(value);
    
    let weiter_button = driver.find(By::Id("WeiterButton")).await?;
    weiter_button.click().await?;

    driver.execute("document.getElementById('OKButton').click();", vec![]).await?;
    dbg!(driver.current_url().await?);

    /* step3 */
    driver.execute("document.querySelector('input[name=\"select_location\"]').click();", vec![]).await?;
    dbg!(driver.current_url().await?);

    /* step4 */
    let text = driver.find(By::Css("h1"))
        .await?
        .text()
        .await?;
    if text.contains("Auswahl der Zeit") {
        dbg!("appointment avaliable");
        let _ = notify(email,&"book_abholung_aufenthaltserlaubnis".to_string());
    } else if text.contains("Keine Zeiten verfügbar") {
        dbg!("appointment not avaliable");
    }

    /* leave */
    driver.quit().await?;
    Ok(())
}

#[tokio::test]
async fn test() {
    let caps = DesiredCapabilities::firefox();
    let driver = WebDriver::new("http://localhost:4444", caps).await.unwrap();
}