# succeed2ban-tui

Made with:
- [ratatui](https://github.com/ratatui-org/ratatui/) 
- [async-template](https://github.com/ratatui-org/ratatui-async-template) 

![VHS](examples/map.gif)
The Map screen, shows the location of IPs from incoming log-in attempts

## Short

Overcomplicated way to tail -f your SSH logs.

Only made for myself to learn Rust, use at own discretion.

Issues / Todos:

1. Does not check for active ssh connections so you might ban yourself

2. Actions need refactor badly

3. Refactor for testing

4. Text wrapping

5. Bans are not correctly logged in the db


Works off the default fail2ban log path @ `/var/log/fail2ban.log`

But you can also set your own by including the configuration file from this repo in ~/.config/succeed2ban-tui/ and setting the `logpath` field at the bottom.


## Usage

1. clone the repo

2. cargo run --release

3. Press `Tab` to skip Startup menu countdown.

4. Start fail2ban and/or journalctl watcher

5. Watch

Press `w` for displaying the help / hotkeys!


![Main](Main_help.PNG)

## About

I once had trouble setting up fail2ban so I had to spent some time looking at logs, so I thought why not spent some more time looking at logs?
I then build a similar app in Python, which was much more limited and limiting. 
After deciding to learn Rust I thought this was a good opportunity to spent even more time looking at logs. So here we are.


succeed2ban-tui monitors journalctl and fail2ban SSH logs. 

It fetches geodata for incoming IPs from [ip-api.com](https://ip-api.com/). 

Stores geodata in a SQLite file in order to keep necessary requests to a minimum and to review log statistics.

Your home IP is fetched from [ident.me](https://ident.me/) for displaying connection lines on map.

### Stat screen

![Stats](Stats.PNG)

Allows for blocking based on:

1. Country
2. Region
3. City
4. ISP

But in the end this is more of an overinflated cMatrix with tail -f on top. So enjoy your CPU cycles :)

Feel free to report any issues you find or suggestions you have!

