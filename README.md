# Manga Discord Bot

## Requirements:

- docker
- sqlx cli
- rust

## Setup

1. Make sure docker is running
2. Run the db setup script

```sh
sh scripts/setup_db.sh
```

3. Run the sqlx command: `sqlx migrate run`
