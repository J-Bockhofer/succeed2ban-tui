# Succed2Ban-tui

Made with [ratatui](https://github.com/ratatui-org/ratatui/) 

## Short

A terminal-based UI for watching your SSH logs and instant access to ban and unban actions.
Gathers statistics and allows for blocking based on:

1. Country
2. Region
3. City
4. ISP

(Not meant for production, just for entertainment)

## VHS 





Watch your logs the right way.

## Usage

1. Build or install

2. Start

3. Start fail2ban and/ journalctl watcher

4. Watch


## Detailed About

I once had trouble setting up fail2ban so I had to spent some time looking at logs, so I thought why not spent some more time looking at logs?
I then build a similar app in Python, which was much more limited and limiting. 
After deciding to learn Rust I thought this was a good opportunity to spent even more time looking at logs. So here we are.

Succeed2Ban monitors journalctl and fail2ban ssh logs. 
It fetches geodata for incoming IPs from [ip-api.com](https://ip-api.com/). 
Stores geodata in a SQLite file in order to keep necessary requests to a minimum and to review log statistics.

But in all honesty, this is more of an overinflated cMatrix. So enjoy your CPU cycles :)




[![CI](https://github.com//ratui/workflows/CI/badge.svg)](https://github.com//ratui/actions)