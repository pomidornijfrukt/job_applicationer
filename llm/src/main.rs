mod llm_client;

use core::str;

use anyhow::Result;
use llm_client::LlmClient;
use regex::Regex;

#[derive(Debug, Clone)]
struct Candidate {
    id: usize,
    path: Vec<String>,
    value: String,
}

fn generate_candidate_dump(candidates: &[Candidate]) -> String {
    candidates
        .iter()
        .map(|c| format!("[{}] {} = {}", c.id, c.path.join("."), c.value))
        .collect::<Vec<_>>()
        .join("\n")
}

fn build_prompt(blob: &str) -> String {
    // let candidate_dump = generate_candidate_dump(candidates);

    format!(
        r##"
Extract the job information.

Return ONLY valid JSON.
Copy information from the input.
If the input does not explicitly contain a value, use null.

{{
  "job_title": null,
  "company": null,
  "location": null,
  "description": null,
  "technologies": null,
  "compensation": null,
  "duration": null,
  "date_posted": null,
  "valid_through": null,
  "employment_type": null
}}

Input:
{blob}

Output:
"##,
    )
}

fn parse_candidates(input: &str) -> Vec<Candidate> {
    let re = Regex::new(r#"^Nuxt candidate (.+?) = (?:String|Number)\((.*)\)$"#).unwrap();

    input
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let line = line.trim();

            let captures = re.captures(line)?;

            let path = captures
                .get(1)?
                .as_str()
                .split('.')
                .map(String::from)
                .collect();

            let value = captures.get(2)?.as_str().trim_matches('"').to_string();

            Some(Candidate {
                id: index + 1,
                path,
                value,
            })
        })
        .collect()
}

#[tokio::main]
async fn main() -> Result<()> {
    let llm = LlmClient::new("http://127.0.0.1:8080");

    let blob = r##"
{"routePath":"/co/PandaRestaurantGroup2/Job","state":{"messenger":{"allowRenderMessage":true,"apiServiceObj":null,"applyJobTerm":false,"candidate":{},"carePrompts":[],"completedRatings":{},"conversationSlugRatingData":{},"currentAnswers":{},"displayedSelectList":false,"eeoQuestion":{"data":null,"disabilityForm":null,"eeoForm":null,"eeoParentChildOptions":false},"isAskingOverallFeedback":false,"isChatBoxOverlay":false,"isComposerFocused":false,"isLoadingRatingData":false,"isRecaptchaReady":false,"isShowingDeclinedTerms":false,"jobAlertIsSubscribing":false,"lastRenderMessage":{},"messageRatingData":null,"messages":[],"pendingMessages":[],"prevCandidateLanguage":null,"previousXhrResp":null,"queued":false,"ratingData":null,"recaptchaToken":"","renderingMessageIds":[],"revertLanguageCode":"","revertLanguageMsgId":0,"revertingLanguage":false,"scrollVal":0,"selectedMediaModalId":null,"sentExternalAdditionalSettings":false,"showRating":false,"urls":[],"ws_token":""},"widget":{"company":{},"extraSettings":{},"firstMinimize":false,"id":null,"interactionState":{"alignment":"right","dialogType":"popover","maximizing":false,"minimizing":false,"state":"open"},"isFirstLoadTrackingPixel":true,"options":{},"rules":null,"visibleExpiredModal":false,"workflowTrackingPixelCallId":null}},"useState":{"job-posting-data":{"ai_info":{"avatar_url":"https://dokumfe7mps0i.cloudfront.net/oms/15330/image/2026/4/RAEOBHV2OQ_17751593484856935/17751593484856935_-1x-1.png","name":"Panda Hiring Assistant"},"company_info":{"id":15330,"logo_url":"https://cdn.olivia.paradox.ai/oms/15330/image/2025/7/852501945351366471_1753341111985/1753341111985_-1x-1.png","name":"Panda Restaurant Group"},"company_settings":{},"composite_slug":"PandaRestaurantGroup2","error":{},"invite_token":"","job_id":"P1-6608928-1","job_info":{"additional_locations":["Support Center-Rosemead"],"address":"1683 Walnut Grove Avenue, Rosemead CA 91770","applicant_flow_id":0,"brand_logo":{"adp_wotc_organization_oid":"","brand_information":"","campaign_names":[],"company_id":15330,"element_color":"#25c9d0","email_header_logo_uri":"https://dokumfe7mps0i.cloudfront.net/oms/15330/image/2026/3/042EE33X2G_17737713986719801/17737713986719801_-1x-1.png","event_names":[],"external_id":"","external_ids":[],"file_name":"Panda Logo.png","id":3253,"is_default":true,"is_from_job_feed":false,"is_used_in_widget":false,"job_names":[],"logo_uri":"https://cdn.olivia.paradox.ai/oms/15330/image/2025/7/852501945351366471_1753341111985/1753341111985_-1x-1.png","name":"Panda Restaurant Group","outbound_phone_numbers":[],"status":1,"updated_at":"2026-03-17T18:16:48.024564Z","widget_names":[]},"conversation_id":1878037,"defined_language_code":"","description":"<p><strong>Summary of Job Description:</strong></p><p>The Software Engineer is responsible for assisting with the implementation of applications or software, ensuring functionality, performance, and security. This position works with databases, APIs, and server-side programming languages to help build robust and scalable systems that support user interfaces</p><p><br></p><p><strong>Job Responsibilities:</strong></p><ul><li>Assists with designing and developing applications, including writing code for applications, building and overseeing databases, and designing and implementing APIs that applications can use to interact with the Users</li><li>Helps ensure application scalability and performance to optimize the application for high traffic and data volumes, ensuring it can handle millions of users and requests without performance degradation.</li><li>Contributes to implementing IS policies, standards, practices, and security measures to ensure effective and consistent information processing operations and to safeguard information resources.&nbsp;Collaborates with IS disciplines to ensure services meet company standards.</li><li>Helps address performance bottlenecks, optimize app loading times, and implement efficient caching strategies to enhance user experience.&nbsp;&nbsp;</li><li>Works on specific production services, helping ensure that they meet the agreed customer requirements for functionality, support, and service levels.</li><li>Acquires knowledge of developments in service areas and helps incorporate new developments into the service areas.</li><li class=\"ql-align-justify\">Works with peers, business partners, offshore development, and vendors to help build and maintain effective relationships and partnerships.</li></ul><p class=\"ql-align-justify\">&nbsp;</p><p class=\"ql-align-justify\"><strong>How we reward you:</strong></p><ul><li>Hybrid remote schedule</li><li>401K with company match</li><li>Yearly bonus opportunity*</li><li>Full medical, dental, and vision insurance *</li><li>On-site fitness center, biometric screen, and flu shot clinic</li><li>Discounts at Panda restaurants, theme parks, and gym memberships</li><li>Paid time off starting at 15 days with 7 federal holidays*</li><li>Continuous education assistance and scholarships*</li><li>Income protection including Disability, Life and AD&amp;D insurance*</li><li>Bereavement leave*</li></ul><p><br></p><p><em>*Benefits available for eligible permanent full time associates</em></p><p><br></p><p><strong>Your background &amp; experience:</strong></p><ul><li>Bachelor’s degree in CS or related field required</li><li>3+ years of professional experience developing scale services which will be used by millions of users, preferably in a retail/hospitality environment</li><li>Experience with programming languages such as C#, Java, and Python</li><li>Proven ability to lead and mentor junior developers</li><li>Strong problem-solving and analytical skills</li><li>Excellent communication and collaboration skills</li><li>Successful completion of initial and periodically required trainings.</li><li>Obtaining a valid Food Handler’s Card within 30 days of employment is a requirement of this position.</li></ul><p><br></p><p>Pay Range: P2H: $86,000 - $121,000 / Annual</p><p>* Within the range, individual pay is determined using various factors, including work location and experience.</p><p>#LI-Hybrid</p><p>#LI-GB1</p><p><br></p><p><p><strong>Panda Strong since 1983:</strong></p><p>Founded in Glendale, California, we are now the largest family-owned American Chinese Restaurant concept in America. With close to 2,800 locations globally, we continue our mission of delivering exceptional Asian dining experiences by building an organization where people are inspired to better their lives. Whether it’s impacting our team or the communities we work in, we’re proud to be an organization that embraces family values.</p><p></p><p><strong>You’re wanted here:</strong></p><p>Panda Restaurant Group, Inc. is an Equal Opportunity Employer and is committed to providing equal opportunity, and does not discriminate on the basis of any characteristic protected by law, including but not limited to sex/gender (including pregnancy, childbirth, lactation and related conditions), gender expression, race, color, religion, national origin, sexual orientation, gender identity, disability, age, ancestry, medical condition, genetic information, marital status, and veteran status. Additionally, Panda Restaurant Group, Inc. complies with all federal, state, and local laws regarding requests for workplace accommodation.&nbsp;The Americans with Disabilities Act (ADA) prohibits discrimination against qualified individuals on the basis of disability.&nbsp;Applicants are entitled to reasonable accommodations, absent undue hardship, to effectively participate in the application and hiring process, for example, sign language interpreters.&nbsp;If you believe you require an accommodation for the application or interview process or for the position for which you are applying, please reach out to<strong>&nbsp;</strong><a target=\"_blank\" rel=\"noopener noreferrer\" href=\"mailto:TASupport@PandaRG.com\"><u>TASupport@PandaRG.com</u></a><u>.</u></p></p><p>&nbsp;</p>","is_internal_job":true,"job_journey_targeting_id":0,"job_loc_id":22622793,"job_posting_type":"External and Internal","job_req_id":"PDX_PRG_2528EC66-8A1C-4C15-A5FC-D94BC05F1DB9","job_token":"PDX_PRG_2528EC66-8A1C-4C15-A5FC-D94BC05F1DB9_22622793","jsr_attributes":[{"order":1,"result_field_type":"job_title","system_attribute_key_id":1115157,"value":"Software Engineer"},{"order":2,"result_field_type":"location_name","system_attribute_key_id":1115167,"value":["Support Center-Rosemead"]}],"language_code":"en","language_codes":[{"active":1,"code":"en","lock":1,"name":"English"}],"live_job_id":6608928,"name":"Software Engineer","paradox_job_type":"Standard Job","posting_job_id":"PDX_PRG_2528EC66-8A1C-4C15-A5FC-D94BC05F1DB9","total_locations":1},"page_settings":{"candidate_language_on":true,"geoloc_allowed":true,"is_ats_job_feed_on":false,"is_cms_on":true,"is_flynn_company":false,"language_code":"en","multi_branding_on":true,"origin_url":"https://olivia.paradox.ai/co/PandaRestaurantGroup2/Job?job_id=P1-6608928-1","redirect_url":"","referer_url":"","request_language_code":"en","show_chat_prompt":1,"show_job_language":true,"site_description":"Join Panda Restaurant Group - in 1683 Walnut Grove Avenue, Rosemead CA 91770 by applying to the Software Engineer job today!","tracking_pixel_script":""},"page_styles":{"btn_color":"25c9d0","chat_prompt":1,"closing_job_button_label":"Browse more jobs","closing_job_header":"Oops!","closing_job_message":"The job you’re looking for is no longer available.","closing_job_search_url":"","fill_color":"f7ffff","header_bg_color":"ffffff","header_text_color":"555555","hot_job_label_color":"#395E66","icons_color":"25c9d0","job_posting_completed_button_text":"Return to Job Posting","job_posting_completed_job_header":"Thank you for completing the screening for #job-title!","job_posting_completed_job_message":"Your screening has been submitted.","job_posting_completed_screening_state_on":1,"job_posting_completed_status_text":"Screening Completed","job_posting_options_apply_button_text":"Chat to Apply","job_posting_options_job_search_text":"View More Jobs","job_posting_options_job_search_url":"","job_posting_options_status_text":"Screening In-Progress","search_jobs_url":"","widget_id":"tkauzhpffyzfllnricgq"},"request_language_code":"en","slug":""}}}
}"##;
    let blobl2 = clean_candidate_blob(blob);
    let candidates = parse_candidates(blob);

    println!("Parsed {} candidates", candidates.len());

    let prompt = build_prompt(blob);

    println!("\n--- PROMPT ---\n");
    println!("{prompt}");

    let answer = llm.chat(&prompt).await?;

    println!("\n--- LLM ANSWER ---\n");
    println!("{answer}");

    Ok(())
}

fn clean_candidate_blob(input: &str) -> String {
    let keep = Regex::new(
        r#"^Nuxt candidate (useState\.job-posting-data(?:\.|$)|state\.|routePath$|layout$)"#,
    )
    .unwrap();
    input
        .lines()
        .map(str::trim)
        .filter(|line| keep.is_match(line))
        .collect::<Vec<_>>()
        .join("\n")
}