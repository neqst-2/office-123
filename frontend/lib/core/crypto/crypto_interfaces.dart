<<<<<<< HEAD
/// Defines a frontend-safe cryptographic envelope contract.
abstract interface class CryptographicEnvelope {
  /// Returns the detached key reference used by the orchestrator.
  String get keyReference;

  /// Returns the base64url-encoded nonce for the protected payload.
  String get nonce;

  /// Returns the base64url-encoded ciphertext payload.
  String get ciphertext;

  /// Returns the authentication tag or MAC for integrity verification.
  String get authenticationTag;
}

/// Defines the UI-facing boundary for future cryptographic helper adapters.
abstract interface class FrontendCryptoFacade {
  /// Validates that a cryptographic envelope is structurally safe to forward.
  ///
  /// Intent:
  /// Enforces a zero-trust boundary in the frontend before encrypted payloads
  /// are handed to lower layers.
  ///
  /// Input/Output constraints:
  /// Accepts a [CryptographicEnvelope] and returns `true` only for envelopes
  /// that meet the structural contract expected by the orchestrator.
  ///
  /// Security implications:
  /// This validation does not replace cryptographic verification; it prevents
  /// obviously malformed data from traversing deeper into privileged layers.
  bool validateEnvelopeShape(final CryptographicEnvelope envelope);
}

abstract interface class CombinedAeadCiphertext {
  String get combined;
}

final class AeadCombinedCiphertext implements CombinedAeadCiphertext {
  const AeadCombinedCiphertext(this.combined);

  @override
  final String combined;

  bool get isNonEmpty => combined.trim().isNotEmpty;
}
=======
/// Defines a frontend-safe cryptographic envelope contract.
abstract interface class CryptographicEnvelope {
  /// Returns the detached key reference used by the orchestrator.
  String get keyReference;

  /// Returns the base64url-encoded nonce for the protected payload.
  String get nonce;

  /// Returns the base64url-encoded ciphertext payload.
  String get ciphertext;

  /// Returns the authentication tag or MAC for integrity verification.
  String get authenticationTag;
}

/// Defines the UI-facing boundary for future cryptographic helper adapters.
abstract interface class FrontendCryptoFacade {
  /// Validates that a cryptographic envelope is structurally safe to forward.
  ///
  /// Intent:
  /// Enforces a zero-trust boundary in the frontend before encrypted payloads
  /// are handed to lower layers.
  ///
  /// Input/Output constraints:
  /// Accepts a [CryptographicEnvelope] and returns `true` only for envelopes
  /// that meet the structural contract expected by the orchestrator.
  ///
  /// Security implications:
  /// This validation does not replace cryptographic verification; it prevents
  /// obviously malformed data from traversing deeper into privileged layers.
  bool validateEnvelopeShape(final CryptographicEnvelope envelope);
}
>>>>>>> 0dc035f57a1c694c8225272cdbd0bfc9c9d60bb9
