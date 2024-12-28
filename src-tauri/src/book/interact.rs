pub use super::*;

/// 和web element的交互
pub trait WebElementInteract {
    /// 这个函数用在需要与某个元素交互前
    fn check(
        &self,
        driver: &WebDriver,
    ) -> impl std::future::Future<Output = WebDriverResult<&WebElement>> + Send;
}

impl WebElementInteract for WebElement {
    async fn check(&self, driver: &WebDriver) -> WebDriverResult<&Self> {
        // dbg!("before check",self.is_clickable().await?);

        // 检查是不是有cookies
        if let Ok(cookie_accept_button) = driver.find(By::Id("cookie_msg_btn_yes")).await {
            let _ = cookie_accept_button.click().await;
        }
        // 检查是否在可视范围内
        let _ = self.scroll_into_view().await;

        // 监测下可不可以点击
        // dbg!("after check",self.is_clickable().await?);
        Ok(self)
    }
}
