// SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import uniffi.qrcloak_bindings.AgeIdentity

fun main() {
    println(AgeIdentity.generate())
}
