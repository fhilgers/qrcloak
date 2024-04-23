package com.github.fhilgers.qrcloak.utils

import android.os.Parcel
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.github.fhilgers.qrcloak.R
import com.github.fhilgers.qrcloak.utils.CompletePayloadParceler.write
import com.github.fhilgers.qrcloak.utils.CompressionSpecParceler.write
import com.github.fhilgers.qrcloak.utils.EncryptionSpecParceler.write
import com.github.fhilgers.qrcloak.utils.IndexParceler.write
import com.github.fhilgers.qrcloak.utils.PartialPayloadHeadParceler.write
import com.github.fhilgers.qrcloak.utils.PartialPayloadParceler.write
import com.github.fhilgers.qrcloak.utils.PartialPayloadTailParceler.write
import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi
import kotlinx.parcelize.Parceler
import uniffi.qrcloak_core.CompletePayload
import uniffi.qrcloak_core.CompressionSpec
import uniffi.qrcloak_core.EncryptionSpec
import uniffi.qrcloak_core.Index
import uniffi.qrcloak_core.PartialPayload
import uniffi.qrcloak_core.PartialPayloadHead
import uniffi.qrcloak_core.PartialPayloadTail
import uniffi.qrcloak_core.Payload

@OptIn(ExperimentalEncodingApi::class)
val Payload.dataString: String
    get() {
        val bytes =
            when (this) {
                is Payload.Complete -> v1.data
                is Payload.Partial ->
                    when (val inner = v1) {
                        is PartialPayload.Head -> inner.v1.data
                        is PartialPayload.Tail -> inner.v1.data
                    }
            }

        return Base64.encode(bytes)
    }

@OptIn(ExperimentalEncodingApi::class)
val CompletePayload.dataString: String
    get() = Base64.encode(data)

@OptIn(ExperimentalEncodingApi::class)
val PartialPayload.dataString: String
    get() {
        val bytes =
            when (this) {
                is PartialPayload.Head -> v1.data
                is PartialPayload.Tail -> v1.data
            }

        return Base64.encode(bytes)
    }

@OptIn(ExperimentalEncodingApi::class)
val List<PartialPayload?>.dataString: String
    get() {
        val bytes =
            this.flatMap {
                    when (it) {
                        is PartialPayload.Head -> it.v1.data.toList()
                        is PartialPayload.Tail -> it.v1.data.toList()
                        else -> listOf()
                    }
                }
                .toByteArray()

        return Base64.encode(bytes)
    }

val PartialPayload.id: UInt?
    get() =
        when (this) {
            is PartialPayload.Head -> v1.index.id
            is PartialPayload.Tail -> null
        }

val List<PartialPayload?>.tag: String
    @Composable get() = stringResource(id = R.string.partial_payloads_tag)

val List<PartialPayload?>.encryptionTag: String
    @Composable
    get() =
        getOrNull(0)?.let {
            if (it is PartialPayload.Head) {
                it.encryptionTag
            } else {
                throw Exception("Misconfigured Payload: Expected Head at position 0")
            }
        } ?: unknownEncryptionTag

val List<PartialPayload?>.id: UInt
    get() =
        mapNotNull {
                when (it) {
                    is PartialPayload.Head -> it.v1.index.id
                    is PartialPayload.Tail -> it.v1.index.id
                    null -> null
                }
            }
            .first()

val List<PartialPayload?>.compressionTag: String
    @Composable
    get() =
        getOrNull(0)?.let {
            if (it is PartialPayload.Head) {
                it.compressionTag
            } else {
                throw Exception("Misconfigured Payload: Expected Head at position 0")
            }
        } ?: unknownCompressionTag

fun CompletePayload.toPayload() = Payload.Complete(this)

fun PartialPayload.toPayload() = Payload.Partial(this)

val CompletePayload.tag: String
    @Composable get() = stringResource(id = R.string.complete_payload_tag)

val PartialPayload.tag: String
    @Composable get() = stringResource(id = R.string.partial_payload_tag)

val EncryptionSpec.tag: String
    @Composable
    get() =
        when (this) {
            EncryptionSpec.NO_ENCRYPTION -> stringResource(id = R.string.encryption_no_encryption)
            EncryptionSpec.AGE_PASSPHRASE -> stringResource(id = R.string.encryption_age_passphrase)
            EncryptionSpec.AGE_KEY -> stringResource(id = R.string.encryption_age_key)
        }

val CompressionSpec.tag: String
    @Composable
    get() =
        when (this) {
            CompressionSpec.NO_COMPRESSION ->
                stringResource(id = R.string.compression_no_compression)
            CompressionSpec.GZIP -> stringResource(id = R.string.compression_gzip)
        }

val CompletePayload.encryptionTag: String
    @Composable get() = encryption.tag

val PartialPayload.encryptionTag: String
    @Composable
    get() =
        when (this) {
            is PartialPayload.Head -> v1.encryption.tag
            is PartialPayload.Tail -> unknownEncryptionTag
        }

val CompletePayload.compressionTag: String
    @Composable get() = compression.tag

val PartialPayload.compressionTag: String
    @Composable
    get() =
        when (this) {
            is PartialPayload.Head -> v1.compression.tag
            is PartialPayload.Tail -> unknownCompressionTag
        }

val unknownCompressionTag
    @Composable get() = stringResource(id = R.string.encryption_unknown)

val unknownEncryptionTag
    @Composable get() = stringResource(id = R.string.compression_unknown)

val Payload.tag: String
    @Composable
    get() =
        when (val t = this) {
            is Payload.Complete -> t.v1.tag
            is Payload.Partial -> t.v1.tag
        }

val Payload.encryptionTag: String
    @Composable
    get() =
        when (val t = this) {
            is Payload.Complete -> t.v1.encryptionTag
            is Payload.Partial -> t.v1.encryptionTag
        }

val Payload.compressionTag: String
    @Composable
    get() =
        when (val t = this) {
            is Payload.Complete -> t.v1.compressionTag
            is Payload.Partial -> t.v1.compressionTag
        }

object EncryptionSpecParceler : Parceler<EncryptionSpec> {
    override fun create(parcel: Parcel): EncryptionSpec =
        EncryptionSpec.valueOf(parcel.readString()!!)

    override fun EncryptionSpec.write(parcel: Parcel, flags: Int) = parcel.writeString(this.name)
}

object CompressionSpecParceler : Parceler<CompressionSpec> {
    override fun create(parcel: Parcel): CompressionSpec =
        CompressionSpec.valueOf(parcel.readString()!!)

    override fun CompressionSpec.write(parcel: Parcel, flags: Int) = parcel.writeString(this.name)
}

object PartialPayloadParceler : Parceler<PartialPayload> {

    enum class VARIANT {
        HEAD,
        TAIL
    }

    override fun create(parcel: Parcel): PartialPayload {
        val variant = VARIANT.entries[parcel.readInt()]

        return when (variant) {
            VARIANT.HEAD -> PartialPayload.Head(PartialPayloadHeadParceler.create(parcel))
            VARIANT.TAIL -> PartialPayload.Tail(PartialPayloadTailParceler.create(parcel))
        }
    }

    override fun PartialPayload.write(parcel: Parcel, flags: Int) {
        when (this) {
            is PartialPayload.Head -> {
                parcel.writeInt(VARIANT.HEAD.ordinal)
                this.v1.write(parcel, flags)
            }
            is PartialPayload.Tail -> {
                parcel.writeInt(VARIANT.TAIL.ordinal)
                this.v1.write(parcel, flags)
            }
        }
    }
}

object OptionalPartialPayloadParceler : Parceler<PartialPayload?> {

    enum class VARIANT {
        SOME,
        NONE
    }

    override fun create(parcel: Parcel): PartialPayload? {
        val variant = VARIANT.entries[parcel.readInt()]

        return when (variant) {
            VARIANT.SOME -> PartialPayloadParceler.create(parcel)
            VARIANT.NONE -> null
        }
    }

    override fun PartialPayload?.write(parcel: Parcel, flags: Int) {
        if (this == null) {
            parcel.writeInt(VARIANT.NONE.ordinal)
        } else {
            parcel.writeInt(VARIANT.SOME.ordinal)

            // WATCH OUT, otherwise recursion
            with(PartialPayloadParceler) { write(parcel, flags) }
        }
    }
}

object PartialPayloadHeadParceler : Parceler<PartialPayloadHead> {
    override fun create(parcel: Parcel): PartialPayloadHead {
        val size = parcel.readInt()
        val data = ByteArray(size)
        parcel.readByteArray(data)

        return PartialPayloadHead(
            data = data,
            compression = CompressionSpecParceler.create(parcel),
            encryption = EncryptionSpecParceler.create(parcel),
            index = IndexParceler.create(parcel)
        )
    }

    override fun PartialPayloadHead.write(parcel: Parcel, flags: Int) {
        parcel.writeInt(this.data.size)
        parcel.writeByteArray(this.data)
        compression.write(parcel, flags)
        encryption.write(parcel, flags)
        index.write(parcel, flags)
    }
}

object PartialPayloadTailParceler : Parceler<PartialPayloadTail> {
    override fun create(parcel: Parcel): PartialPayloadTail {
        val size = parcel.readInt()
        val data = ByteArray(size)
        parcel.readByteArray(data)

        return PartialPayloadTail(data = data, index = IndexParceler.create(parcel))
    }

    override fun PartialPayloadTail.write(parcel: Parcel, flags: Int) {
        parcel.writeInt(this.data.size)
        parcel.writeByteArray(this.data)
        index.write(parcel, flags)
    }
}

object IndexParceler : Parceler<Index> {
    override fun create(parcel: Parcel): Index =
        Index(
            id = parcel.readInt().toUInt(),
            index = parcel.readInt().toUInt(),
            size = parcel.readInt().toUInt()
        )

    override fun Index.write(parcel: Parcel, flags: Int) {
        parcel.writeInt(this.id.toInt())
        parcel.writeInt(this.index.toInt())
        parcel.writeInt(this.size.toInt())
    }
}

object CompletePayloadParceler : Parceler<CompletePayload> {
    override fun create(parcel: Parcel): CompletePayload {
        val size = parcel.readInt()
        val data = ByteArray(size)
        parcel.readByteArray(data)

        return CompletePayload(
            data = data,
            encryption = EncryptionSpecParceler.create(parcel),
            compression = CompressionSpecParceler.create(parcel)
        )
    }

    override fun CompletePayload.write(parcel: Parcel, flags: Int) {
        parcel.writeInt(this.data.size)
        parcel.writeByteArray(this.data)
        this.encryption.write(parcel, flags)
        this.compression.write(parcel, flags)
    }
}
