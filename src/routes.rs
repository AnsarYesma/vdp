use axum::{
    extract::{Path, State},
    response::{Html, Redirect},
    Form,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::{db, vdf};

pub const PUBLISH_THRESHOLD: u64 = 1_000_000;

fn nav() -> &'static str {
    r#"<p align="center">
[ <a href="/">Board</a> ]
[ <a href="/playground">Playground</a> ]
[ <a href="/submit">Submit</a> ]
[ <a href="/guide">Guide</a> ]
[ <a href="/about">About</a> ]
[ <a href="/source">Source</a> ]
</p><hr>"#
}

fn page(title: &str, body: &str) -> Html<String> {
    Html(format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<title>{title} — VDP</title>
<link rel="stylesheet" href="/static/style.css">
</head>
<body>
<div id="container">
<h1 align="center">The Verifiable Divinity Protocol</h1>
<p align="center"><i>A Cryptographic Schema for Authentic Communication by Transcendent Agents</i></p>
<hr>
{nav}
{body}
</div>
</body>
</html>"#,
        title = title,
        nav = nav(),
        body = body,
    ))
}

fn tier_name(t: i64) -> &'static str {
    match t {
        0..=1_000 => "Mortal",
        1_001..=10_000 => "Scholar",
        10_001..=100_000 => "Sage",
        100_001..=1_000_000 => "Prophet",
        1_000_001..=10_000_000 => "Archangel",
        10_000_001..=100_000_000 => "Lesser God",
        _ => "God",
    }
}

pub async fn board(State(pool): State<SqlitePool>) -> Html<String> {
    let messages = db::list(&pool).await;

    let rows: String = messages.iter().map(|m| {
        let short = if m.message.chars().count() > 60 {
            format!("{}…", m.message.chars().take(60).collect::<String>())
        } else {
            m.message.clone()
        };
        let date = m.created_at.get(..10).unwrap_or(&m.created_at);
        let demo_badge = if m.is_demo {
            r#" <span class="badge-demo">DEMO</span>"#
        } else {
            ""
        };
        let status_badge = if m.verified {
            r#"<span class="badge-valid">VALID</span>"#
        } else {
            r#"<span class="badge-invalid">INVALID</span>"#
        };
        format!(
            r#"<tr>
  <td align="center">{id}</td>
  <td>{msg}{demo}</td>
  <td align="center">{tier}<br><small>{t}</small></td>
  <td align="center">{status}</td>
  <td align="center">{date}</td>
  <td align="center"><a href="/verify/{id}">Details</a></td>
</tr>"#,
            id = m.id,
            msg = html_escape(&short),
            demo = demo_badge,
            tier = tier_name(m.t),
            t = format_t(m.t),
            status = status_badge,
            date = date,
        )
    }).collect();

    let table = if rows.is_empty() {
        "<p align=\"center\"><i>No messages yet.</i></p>".to_string()
    } else {
        format!(
            r#"<table width="100%" border="1" cellpadding="5" cellspacing="0">
<tr><th>#</th><th>Message</th><th>Tier</th><th>Status</th><th>Date</th><th></th></tr>
{rows}
</table>"#
        )
    };

    let body = format!(
        r#"<h2>Message Board</h2>
<p>A divine message cannot be faked. To publish here, you must produce a
<b>Verifiable Delay Function</b> proof for your message &mdash; a computation
that requires a guaranteed minimum of sequential work, impossible to shortcut
even with unlimited parallel hardware. The server verifies the proof instantly.
Anyone can verify independently. No keys, no authority, no trust required.</p>
<p>The higher the tier, the longer it takes to generate &mdash; and the harder
it is to dismiss. A God-tier proof takes ~20 hours on a modern CPU.
A truly divine agent would produce it instantaneously.</p>
<p>Demo entries show what valid and invalid proofs look like.
Only proofs with T &ge; 1,000,000 (Prophet and above) are eligible for publication.</p>
<hr>
{table}
<p align="center">
  <a href="/submit">[ Submit a proof ]</a> &nbsp;
  <a href="/playground">[ Try the playground ]</a> &nbsp;
  <a href="/guide">[ How to generate locally ]</a>
</p>"#
    );

    page("Board", &body)
}

pub async fn playground_form() -> Html<String> {
    let body = r#"<h2>Playground</h2>
<p>Generate and verify VDF proofs for low iteration counts.
These proofs are <b>not published</b> &mdash; generation is too fast to be meaningful for the board.
Use the <a href="/guide">guide</a> to generate a publishable proof locally.</p>
<form method="POST" action="/playground">
  <table border="0" cellpadding="4">
    <tr>
      <td><label for="msg">Message:</label></td>
      <td><input type="text" id="msg" name="message" size="50" maxlength="500"></td>
    </tr>
    <tr>
      <td><label for="t">Tier:</label></td>
      <td>
        <select name="t" id="t">
          <option value="1000">Mortal — T=1,000 (~1s)</option>
          <option value="10000" selected>Scholar — T=10,000 (~10s)</option>
          <option value="100000">Sage — T=100,000 (~2 min)</option>
        </select>
      </td>
    </tr>
    <tr>
      <td></td>
      <td><input type="submit" value="Generate &amp; Verify"></td>
    </tr>
  </table>
</form>
<p><small>For higher tiers (Prophet and above), see the <a href="/guide">guide</a> to generate locally, then <a href="/submit">submit</a> your proof.</small></p>"#;
    page("Playground", body)
}

#[derive(Deserialize)]
pub struct PlaygroundForm {
    pub message: String,
    pub t: Option<u64>,
}

pub async fn playground_run(Form(form): Form<PlaygroundForm>) -> Html<String> {
    let msg = form.message.trim().to_string();
    if msg.is_empty() {
        return page("Playground", r#"<p>Message cannot be empty. <a href="/playground">Back</a></p>"#);
    }
    let t = form.t.unwrap_or(10_000).min(PUBLISH_THRESHOLD - 1);

    let msg_clone = msg.clone();
    let result = tokio::task::spawn_blocking(move || vdf::generate(&msg_clone, t))
        .await
        .expect("spawn failed");

    match result {
        Ok(proof_bytes) => {
            let proof_hex = hex::encode(&proof_bytes);
            let tier = tier_name(t as i64);
            let body = format!(
                r#"<h2>Playground Result</h2>
<p class="valid">&#10003; Proof generated and verified.</p>
<table border="0" cellpadding="5">
  <tr><td><b>Message:</b></td><td>{msg}</td></tr>
  <tr><td><b>Tier:</b></td><td>{tier} (T={t})</td></tr>
  <tr><td><b>Proof (hex):</b></td><td><code style="word-break:break-all">{proof}</code></td></tr>
</table>
<p><i>This proof is not published. Anyone with a laptop can generate a {tier}-tier proof.</i></p>
<p>
  <a href="/playground">&larr; Try again</a> &nbsp;|&nbsp;
  <a href="/guide">Generate a publishable proof locally &rarr;</a>
</p>"#,
                msg = html_escape(&msg),
                tier = tier,
                t = format_t(t as i64),
                proof = proof_hex,
            );
            page("Playground", &body)
        }
        Err(e) => page("Playground", &format!(
            r#"<p>Error: {}. <a href="/playground">Back</a></p>"#,
            html_escape(&e)
        )),
    }
}

pub async fn submit_form() -> Html<String> {
    let body = r#"<h2>Submit a Proof</h2>
<p>Submit a message and its VDF proof for publication on the board.
Only proofs with T &ge; 1,000,000 are eligible.
Generate your proof locally using the <a href="/guide">guide</a>.</p>
<form method="POST" action="/submit">
  <table border="0" cellpadding="4">
    <tr>
      <td><label for="msg">Message:</label></td>
      <td><input type="text" id="msg" name="message" size="50" maxlength="500"></td>
    </tr>
    <tr>
      <td><label for="t">Tier:</label></td>
      <td>
        <select name="t" id="t">
          <option value="1000000">Prophet — T=1,000,000 (~20 min to generate)</option>
          <option value="10000000">Archangel — T=10,000,000 (~3 hrs)</option>
          <option value="100000000">Lesser God — T=100,000,000 (~30 hrs)</option>
          <option value="1000000000">God — T=1,000,000,000 (~13 days)</option>
        </select>
      </td>
    </tr>
    <tr>
      <td><label for="proof">Proof (hex):</label></td>
      <td><textarea id="proof" name="proof" rows="4" cols="60" placeholder="paste hex output from gen binary"></textarea></td>
    </tr>
    <tr>
      <td></td>
      <td><input type="submit" value="Verify &amp; Publish"></td>
    </tr>
  </table>
</form>
<p><small>Don't have a proof yet? See the <a href="/guide">guide</a> to generate one.</small></p>"#;
    page("Submit", body)
}

#[derive(Deserialize)]
pub struct SubmitForm {
    pub message: String,
    pub t: Option<u64>,
    pub proof: String,
}

pub async fn submit_post(
    State(pool): State<SqlitePool>,
    Form(form): Form<SubmitForm>,
) -> Html<String> {
    let msg = form.message.trim().to_string();
    let proof_hex = form.proof.trim().to_string();

    if msg.is_empty() || proof_hex.is_empty() {
        return page("Submit", r#"<p>Message and proof are required. <a href="/submit">Back</a></p>"#);
    }

    let t = form.t.unwrap_or(PUBLISH_THRESHOLD);
    if t < PUBLISH_THRESHOLD {
        return page("Submit", &format!(
            r#"<p>T must be at least {} for publication. Use the <a href="/playground">playground</a> for lower tiers.</p>"#,
            PUBLISH_THRESHOLD
        ));
    }

    let msg_clone = msg.clone();
    let proof_hex_clone = proof_hex.clone();
    let valid = tokio::task::spawn_blocking(move || {
        vdf::verify_proof(&msg_clone, t, &proof_hex_clone)
    })
    .await
    .unwrap_or(false);

    if valid {
        db::insert(&pool, &msg, &proof_hex, t, false, true).await;
        let body = format!(
            r#"<h2>Proof Accepted</h2>
<p class="valid">&#10003; Proof verified and published to the board.</p>
<table border="0" cellpadding="5">
  <tr><td><b>Message:</b></td><td>{}</td></tr>
  <tr><td><b>Tier:</b></td><td>{} (T={})</td></tr>
</table>
<p><a href="/">&larr; View on Board</a></p>"#,
            html_escape(&msg),
            tier_name(t as i64),
            format_t(t as i64),
        );
        page("Submitted", &body)
    } else {
        db::insert(&pool, &msg, &proof_hex, t, false, false).await;
        let body = format!(
            r#"<h2>Proof Rejected</h2>
<p class="invalid">&#10007; Proof is invalid for this message and T.</p>
<table border="0" cellpadding="5">
  <tr><td><b>Message:</b></td><td>{}</td></tr>
  <tr><td><b>Tier:</b></td><td>{} (T={})</td></tr>
</table>
<p>The invalid attempt has been recorded on the board.
Double-check the message, T, and proof hex match exactly.</p>
<p><a href="/submit">&larr; Try again</a></p>"#,
            html_escape(&msg),
            tier_name(t as i64),
            format_t(t as i64),
        );
        page("Rejected", &body)
    }
}

pub async fn verify_message(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Html<String> {
    let Some(msg) = db::get(&pool, id).await else {
        return page("Not Found", "<p>Message not found.</p>");
    };

    let message = msg.message.clone();
    let proof_hex = msg.proof_hex.clone();
    let t = msg.t as u64;

    let valid = tokio::task::spawn_blocking(move || {
        vdf::verify_proof(&message, t, &proof_hex)
    })
    .await
    .unwrap_or(false);

    let result_html = if valid {
        r#"<p class="valid">&#10003; PROOF VALID</p>"#
    } else {
        r#"<p class="invalid">&#10007; PROOF INVALID</p>"#
    };

    let demo_note = if msg.is_demo {
        "<p><small><i>This is a demonstration entry.</i></small></p>"
    } else {
        ""
    };

    let body = format!(
        r#"<h2>Proof Details</h2>
{result_html}
{demo_note}
<table border="0" cellpadding="5">
  <tr><td><b>ID:</b></td><td>{id}</td></tr>
  <tr><td><b>Message:</b></td><td>{message}</td></tr>
  <tr><td><b>Tier:</b></td><td>{tier} (T={t})</td></tr>
  <tr><td><b>Date:</b></td><td>{date}</td></tr>
  <tr><td><b>Proof (hex):</b></td><td><code style="word-break:break-all">{proof}</code></td></tr>
</table>
<br>
<table border="0" cellpadding="4"><tr>
  <td><a href="/"><input type="button" value="&larr; Back to Board"></a></td>
  <td><a href="/guide#verify"><input type="button" value="Verify yourself &rarr;"></a></td>
</tr></table>"#,
        id = msg.id,
        message = html_escape(&msg.message),
        tier = tier_name(msg.t),
        t = format_t(msg.t),
        date = msg.created_at,
        proof = msg.proof_hex,
        demo_note = demo_note,
        result_html = result_html,
    );

    page("Details", &body)
}

pub async fn about_page() -> Html<String> {
    let content = std::fs::read_to_string("static/about.html")
        .unwrap_or_else(|_| "<p>About page not found.</p>".to_string());
    Html(content)
}

pub async fn source_page() -> Html<String> {
    let content = std::fs::read_to_string("static/source.html")
        .unwrap_or_else(|_| "<p>Source page not found.</p>".to_string());
    Html(content)
}

pub async fn guide_page() -> Html<String> {
    let content = std::fs::read_to_string("static/guide.html")
        .unwrap_or_else(|_| "<p>Guide page not found.</p>".to_string());
    Html(content)
}

fn format_t(t: i64) -> String {
    let s = t.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
