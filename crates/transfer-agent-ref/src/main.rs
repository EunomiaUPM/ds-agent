/*
 *
 *  * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;
use transfer_agent_ref::setup::cmd::TransferCommands;
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

Starting Eunomia DS-Agent Transfer Agent Server 🌈🌈
UPM Dataspace agent
Show some love on https://github.com/EunomiaUPM/ds-agent
----------

";

#[allow(clippy::result_large_err)]
#[tokio::main]
async fn main() -> Outcome<()> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse("debug,sqlx::query=off")
        .map_err(|e| Errors::crazy(e.to_string(), Some(Box::new(e))))?;
    tracing_subscriber::fmt()
        .event_format(tracing_subscriber::fmt::format().with_line_number(true))
        .with_env_filter(filter)
        .init();
    info!("{}", INFO);
    TransferCommands::init_command_line().await?;
    Ok(())
}
