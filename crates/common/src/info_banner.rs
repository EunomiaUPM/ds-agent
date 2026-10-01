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

Starting Eunomia DS-Agent {replace} Server 🌈🌈
UPM Dataspace agent
Show some love on https://github.com/EunomiaUPM/ds-agent
----------

";

pub fn banner(service_name: &str) -> String {
    let out = INFO.replace("{replace}", service_name);
    out
}
