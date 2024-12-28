use super::*;
use anyhow::{Ok, Result};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

pub fn notify(uid: uuid::Uuid, message: String) -> Result<()> {
    let app = get_app_ins()?;
    let host_email_account = app.get_config_value("host_email_account");
    let host_email_password = app.get_config_value("host_email_password");

    let status = app.get_command_status(uid)?;
    let user_email_account = match status {
        CommandStatus::Book { user_info, .. } => user_info.email.clone(),
        _ => panic!(),
    };

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

    let creds = Credentials::new(
        host_email_account.to_owned(),
        host_email_password.to_owned(),
    );

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    if let Err(r) = mailer.send(&email) {
        log::trace!("{}", &r);
        return Err(r.into());
    }
    Ok(())
}

#[test]
fn test() {
    // let email = "zhouyi.guan@rwth-aachen.de".to_string();
    // let message = "12345".to_string();
    // let _ = notify(email, message);
}
