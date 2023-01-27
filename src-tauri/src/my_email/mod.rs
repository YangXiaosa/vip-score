use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use crate::my_config;

pub fn send_mail(subject: &str, msg: &str) {
    let admin_email = my_config::get_admin_email();
    let admin_email_password = my_config::get_admin_email_password();
    let admin_email_server = my_config::get_admin_email_server();
    let merchant_name = my_config::get_merchant_name();
    if admin_email == "" || admin_email_password == "" || admin_email_server == "" || merchant_name == "" {
        log::error!("send mail failed admin_email:{},admin_email_password:{},admin_email_server:{},merchant_name:{}!!!"
            , admin_email, admin_email_password, admin_email_server, merchant_name);
        return;
    }
    let from = format!("{} <{}>", merchant_name, admin_email);
    let to = format!("管理员 <{}>", admin_email);
    let email = Message::builder()
        .from(from.parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .body(String::from(msg))
        .unwrap();
    let credentials = Credentials::new(admin_email.to_string(), admin_email_password.to_string());
    // Open a remote connection to mail server
    let mailer = SmtpTransport::relay(admin_email_server).unwrap().credentials(credentials).build();
    // Send the email
    match mailer.send(&email) {
        Ok(_) => log::info!("success send mail, subject:{}, msg:{}", subject, msg),
        Err(e) => log::error!("failed send mail, email:{}, server:{}, password:{}, subject:{}, msg:{}, e:{}", admin_email, admin_email_server, admin_email_password, subject, msg, e),
    }
}