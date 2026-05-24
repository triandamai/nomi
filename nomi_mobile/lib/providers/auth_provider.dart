import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:nomi_mobile/data/models/conversation.dart';
import 'package:nomi_mobile/providers/repositories.dart';

class AuthState {
  final Profile? user;
  final bool isLoading;
  final String? error;
  final bool isAuthenticated;
  final bool otpSent;
  final String? identity;

  AuthState({
    this.user,
    this.isLoading = false,
    this.error,
    this.isAuthenticated = false,
    this.otpSent = false,
    this.identity,
  });

  AuthState copyWith({
    Profile? user,
    bool? isLoading,
    String? error,
    bool? isAuthenticated,
    bool? otpSent,
    String? identity,
  }) {
    return AuthState(
      user: user ?? this.user,
      isLoading: isLoading ?? this.isLoading,
      error: error,
      isAuthenticated: isAuthenticated ?? this.isAuthenticated,
      otpSent: otpSent ?? this.otpSent,
      identity: identity ?? this.identity,
    );
  }
}

class AuthNotifier extends Notifier<AuthState> {
  final _storage = const FlutterSecureStorage();


  @override
  AuthState build() {
    _checkAuth();
    return AuthState(isLoading: false, isAuthenticated: false);
  }

  Future<void> _checkAuth() async {
    final token = await _storage.read(key: 'jwt_token');
    if (token == null) {
      state = state.copyWith(isAuthenticated: false, isLoading: false);
      return;
    }

    state = state.copyWith(isLoading: true);
    try {
      final response = await ref.read(authRepositoryProvider).getProfile();
      if (response.meta.isSuccess) {
        state = state.copyWith(
          user: response.data,
          isAuthenticated: true,
          isLoading: false,
        );
      } else {
        await logout();
      }
    } catch (e) {
      state = state.copyWith(isLoading: false, isAuthenticated: false, error: e.toString());
    }
  }

  Future<bool> requestOtp(String identity, String channel) async {
    state = state.copyWith(isLoading: true, identity: identity);
    try {
      final response = await ref.read(authRepositoryProvider).requestOtp(identity, channel);
      state = state.copyWith(isLoading: false, otpSent: response.meta.isSuccess);
      return response.meta.isSuccess;
    } catch (e) {
      state = state.copyWith(isLoading: false, error: e.toString());
      return false;
    }
  }

  Future<bool> verifyOtp(String identity, String code) async {
    state = state.copyWith(isLoading: true);
    try {
      final response = await ref.read(authRepositoryProvider).verifyOtp(identity, code);
      if (response.meta.isSuccess && response.data != null) {
        final token = response.data!['access_token'] as String;
        final userId = response.data!['user_id'] as String;
        await _storage.write(key: 'jwt_token', value: token);
        await _storage.write(key: 'user_id', value: userId);
        await _checkAuth();
        return true;
      }
      state = state.copyWith(isLoading: false, error: response.meta.message);
      return false;
    } catch (e) {
      state = state.copyWith(isLoading: false, error: e.toString());
      return false;
    }
  }

  Future<bool> updateProfile(String displayName) async {
    try {
      final response = await ref.read(authRepositoryProvider).updateProfile(displayName);
      if (response.meta.isSuccess) {
        state = state.copyWith(
          user: Profile(
            id: state.user!.id,
            externalId: state.user!.externalId,
            displayName: displayName,
            avatarUrl: state.user!.avatarUrl,
            role: state.user!.role,
          ),
        );
        return true;
      }
      return false;
    } catch (e) {
      state = state.copyWith(error: e.toString());
      return false;
    }
  }

  void resetOtp() {
    state = state.copyWith(otpSent: false, identity: null);
  }

  Future<void> logout() async {
    await _storage.delete(key: 'jwt_token');
    await _storage.delete(key: 'user_id');
    state = AuthState();
  }
}

final authProvider = NotifierProvider<AuthNotifier, AuthState>(() {
  return AuthNotifier();
});
