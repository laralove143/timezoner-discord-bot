# timezoner

[![add it to your server - invite](https://img.shields.io/badge/add_it_to_your_server-invite-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://discord.com/api/oauth2/authorize?client_id=909820903574106203&permissions=0&scope=bot%20applications.commands)  
[![talk to me - join server](https://img.shields.io/badge/talk_to_me-join-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/RQhskPjrGv)

goodbye to timezone conversions! this bot tries to make sharing times/dates with your friends all over the world easier

## commands
### /timezone [timezone]
yes you need this because time sucks, it's just a one-time copy-paste from [a timezone picker](https://kevinnovak.github.io/Time-Zone-Picker) though
### /time [hour] [minute] (am_pm) (day) (month) (year)
give it an hour and give it a minute, in am-pm or in 24-hour, maybe it a day, maybe a month, maybe a year, and it will send a magical little message that shows that time or date in everyone's own timezone!

## nerdy stuff
made with [Rust](https://www.rust-lang.org) and [Twilight](https://github.com/twilight-rs/twilight) and SQLite.. basically meaning it's good™️
### self-hosting (builds avaliable for linux only)
set `TIMEZONER_BOT_TOKEN` environment variable to your bot's token, [get the latest release](https://github.com/laralove143/timezoner-discord-bot/releases/latest), extract it and run it with `./timezoner` (it takes a few hours for the commands to be registered, be patient)

*made by me (laralove143), licensed MIT*
