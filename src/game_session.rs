use serde::{Deserialize, Serialize, Serializer};
use serde::ser::SerializeTuple;
use crate::error::CodingGameError;

struct PuzzleSessionRequestData {
    unk1: Option<()>,
    name: String,
    unk2: bool,
}

impl PuzzleSessionRequestData {
    fn new(name : String) -> Self {
        Self{
            unk1: None,
            name,
            unk2: false,
        }
    }
}

impl Serialize for PuzzleSessionRequestData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut seq = serializer.serialize_tuple(3)?;
        seq.serialize_element(&self.unk1)?;
        seq.serialize_element(&self.name)?;
        seq.serialize_element(&self.unk2)?;
        seq.end()
    }
}

#[derive(Debug, Deserialize)]
struct PuzzleSessionRequestResponse {
    //#[serde(rename = "reportReady")]
    //report_ready: bool,
    handle: String,
    //direct: bool,
}



struct TestSessionRequestData {
    session_id: String,
}
impl Serialize for crate::game_session::TestSessionRequestData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut seq = serializer.serialize_tuple(1)?;
        seq.serialize_element(&self.session_id)?;
        seq.end()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct  TestCase {
    _index : usize,
    input_binary_id : usize,
    output_binary_id : usize,
    label : String,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct  AvailableLanguage {
    _id : String,
    _name : String
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    title : String,
    statement: String,
    test_cases : Vec<TestCase>,
    _available_languages : Vec<AvailableLanguage>,
    stub_generator : String
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct  CurrentQuestion {
    question : Question
}
#[derive(Debug, Deserialize)]
struct TestSessionRequestResponse {
    #[serde(rename = "currentQuestion")]
    current_question: CurrentQuestion,
}


#[derive(Debug, Serialize)]
pub struct GameDataTest {
    pub label: String,
    pub safe_label: String,
    pub input_text : String,
    pub output_text : String
}


#[derive(Debug, Serialize)]
pub struct GameData {
    pub safe_name: String,
    pub title : String,
    pub description : String,
    pub tests : Vec<GameDataTest>,
    pub stub_generator: String
}

fn safe_name(name : &str) -> String {
    let filtered = name.chars().filter(|c| c.is_alphabetic() || c.is_numeric() || "-_ ".contains(*c) ).collect::<String>();
    filtered.to_lowercase()
        .trim_start_matches(char::is_numeric)
        .replace(['-', ' '], "_")
}

impl GameData {
    async fn new(question : Question ) -> Result<Self, CodingGameError> {
        let mut tests = Vec::new();
        for (test_id, test) in question.test_cases.iter().enumerate() {
            let input = get_servlet_file(test.input_binary_id).await?;
            let output = get_servlet_file(test.output_binary_id).await?;

            tests.push(GameDataTest{
                label: test.label.clone(),
                safe_label: format!("test_{:02}", test_id+1),
                input_text: input,
                output_text: output
            })
        }

        let description = mdka::from_html(&question.statement);

        Ok(Self { safe_name: "".to_string(), title: question.title, description, tests, stub_generator: question.stub_generator })
    }

    pub fn set_safe_name(&mut self, name : &str) {
        self.safe_name = safe_name(name);
    }

}

pub async fn create_game_session(name : String) -> Result<String, CodingGameError> {
    let url = "https://www.codingame.com/services/Puzzle/generateSessionFromPuzzlePrettyId";

    let data = serde_json::to_string(&PuzzleSessionRequestData::new(name))?;

    let client = reqwest::Client::new();

    let response = client.post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(data.clone())
        .send()
        .await?;

    let session_id = response.json::<PuzzleSessionRequestResponse>().await?.handle;

    Ok(session_id)
}

pub async fn get_session_data(session_id: String) -> Result<GameData, CodingGameError> {
    let url = "https://www.codingame.com/services/TestSession/startTestSession";
    let data = serde_json::to_string(&TestSessionRequestData{session_id})?;

    let client = reqwest::Client::new();

    let response = client.post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(data.clone())
        .send()
        .await?;

    let question = response.json::<TestSessionRequestResponse>().await?.current_question.question;

    let game_data = GameData::new(question).await?;

    Ok(game_data)
}


pub async fn get_servlet_file(file_id: usize) -> Result<String, CodingGameError> {
    let url = format!("https://static.codingame.com/servlet/fileservlet?id={file_id}");
    let client = reqwest::Client::new();
    let response = client.get(url)
        .send()
        .await?;

    let res = response.text().await?;

    Ok(res)
}
