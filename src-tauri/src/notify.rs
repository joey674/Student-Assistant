use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use anyhow::{Ok, Result};
use super::*;

pub fn notify(user_email_account: String, message: String) -> Result<()>{
    let host_email_account = CONFIG.get("host_email_account").unwrap().as_str().unwrap();
    let host_email_password = CONFIG.get("host_email_password").unwrap().as_str().unwrap();

    let email = Message::builder()
        .from(host_email_account.parse().unwrap())
        .to(user_email_account.parse().unwrap())
        .subject("Appointment Availiable Notification")
        .header(ContentType::TEXT_PLAIN)
        .body(format!(
            "Dear User,\n\n\
            I am pleased to inform you that an appointment is now available!\n\n\
            Details:\n\
            {}\n\n\
            Best regards,\n\
            ZY",
            message
        ))
        .unwrap();

    let creds = Credentials::new(host_email_account.to_owned(), host_email_password.to_owned());

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    if let Err(r) = mailer.send(&email) {
        log::trace!("{}",&r);
        return Err(r.into());
    }
    Ok(())
}

#[test]
fn test() {
    let email = "zhouyi.guan@rwth-aachen.de".to_string();
    let message = "12345".to_string();
    let _ = notify(email,message);
}
