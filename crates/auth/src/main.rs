/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use auth::setup::cmd::AuthCommands;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use ymir::errors::{Errors, Outcome};

const INFO: &str = r"
----------
:::::::::: :::    ::: ::::    :::  ::::::::  ::::    ::::  :::::::::::     :::
:+:        :+:    :+: :+:+:   :+: :+:    :+: +:+:+: :+:+:+     :+:       :+: :+:
+:+        +:+    +:+ :+:+:+  +:+ +:+    +:+ +:+ +:+:+ +:+     +:+      +:+   +:+
+#++:++#   +#+    +:+ +#+ +:+ +#+ +#+    +:+ +#+  +:+  +#+     +#+     +#++:++#++:
+#+        +#+    +#+ +#+  +#+#+# +#+    +#+ +#+       +#+     +#+     +#+     +#+
#+#        #+#    #+# #+#   #+#+# #+#    #+# #+#       #+#     #+#     #+#     #+#
##########  ########  ###    ####  ########  ###       ### ########### ###     ###
:::::::::   ::::::::                    :::      ::::::::  :::::::::: ::::    ::: :::::::::::
:+:    :+: :+:    :+:                 :+: :+:   :+:    :+: :+:        :+:+:   :+:     :+:
+:+    +:+ +:+                       +:+   +:+  +:+        +:+        :+:+:+  +:+     +:+
+#+    +:+ +#++:++#++ +#++:++#++:++ +#++:++#++: :#:        +#++:++#   +#+ +:+ +#+     +#+
+#+    +#+        +#+               +#+     +#+ +#+   +#+# +#+        +#+  +#+#+#     +#+
#+#    #+# #+#    #+#               #+#     #+# #+#    #+# #+#        #+#   #+#+#     #+#
#########   ########                ###     ###  ########  ########## ###    ####     ###

Starting Eunomia DS-Agent Auth Server 🌈🌈
UPM Dataspace multistack agent
Show some love on https://github.com/EunomiaUPM/ds-agent
----------

";

#[tokio::main]
async fn main() -> Outcome<()> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse("debug,sqlx::query=off")
        .map_err(|e| {
            let error = Errors::crazy("Unexpected error on main", Some(Box::new(e)));
            error.log();
            error
        })?;
    tracing_subscriber::fmt().with_env_filter(filter).init();
    info!("{}", INFO);
    AuthCommands::init_command_line().await?;
    Ok(())
}
