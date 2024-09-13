# Slack Events API Server

This Rust application implements a server that listens for events from the Slack Events API. It uses the `slack-morphism` crate to handle interactions with the Slack API. This Slack server is currently equipped to handle any message-related events as well as reactions. It uses a Mutex queue to avoid losing packages. There are also functions useful for sending messages and interacting with the Web API from Slack.

## Requirements

- Rust programming language and Cargo build system installed
- Ngrok or similar tunneling service (optional, for exposing the local server to the internet)

## Setup 

1. Clone this repository to your local machine.

2. Install NGROk
    - Navigate to https://ngrok.com/ and create an account 
    - Follow their installation process and add te authentication token 

3. Collect Workspace Token 
    - Navigate to "https://api.slack.com/apps"
    - Login using your slack account credentials 
    - In the section title "Your App Configuration Tokens"
        * Click on "Generate Token"
        * Select desired workspace 
        * Click on "Generate"
        * Under Access token for your desired workspace, click "copy" 

4. Place token in .env file under "SLACK_CONFIG_TOKEN" 

5. Run command "cargo run" 

6. A OAuthentication URL will be printed to console, Navigate to it 

7. Click "Allow" to allow the app to be installed on your workspace. 

8. If app was successfully installed, then the event server should be listening for Slack events 


If events are not printing on console, navigate to your apps event subscriptions under features in the left naviagtion pane. 
Click on the Verify button and the url should say verified. Now events will start being sent. 

