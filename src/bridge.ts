import { invoke, isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

export type Settings = {
  sendMode: 'on_hotkey' | 'automatic';
  responseMode: 'suggested_answers' | 'summary' | 'custom';
  customInstruction: string;
  overlayShortcut: string;
  sendShortcut: string;
  launchOnLogin: boolean;
};

export type AppState = {
  settings: Settings;
  audioAvailable: boolean;
  chatgptConnected: boolean;
  inferenceAvailable: boolean;
  shortcutError: string | null;
};

export type TranscriptLine = { id: number; source: 'system' | 'microphone'; startMs: number; text: string };
export type MeetingState = {
  status: 'idle' | 'starting' | 'active' | 'stopping' | 'error';
  meetingId: number | null;
  title: string;
  startedAt: number | null;
  transcript: TranscriptLine[];
  error: string | null;
};

export const desktop = isTauri();
export const defaults: Settings = {
  sendMode: 'on_hotkey', responseMode: 'suggested_answers', customInstruction: '',
  overlayShortcut: 'Ctrl+Space', sendShortcut: 'Ctrl+Shift+Enter', launchOnLogin: false,
};
const previewKey = 'harness-interface-preview-settings';

export async function loadState(): Promise<AppState> {
  if (desktop) return invoke<AppState>('get_app_state');
  const saved = localStorage.getItem(previewKey);
  return { settings: saved ? { ...defaults, ...JSON.parse(saved) } : defaults, audioAvailable: false, chatgptConnected: false, inferenceAvailable: false, shortcutError: null };
}

export async function saveSettings(settings: Settings): Promise<Settings> {
  if (desktop) return invoke<Settings>('save_settings', { settings });
  localStorage.setItem(previewKey, JSON.stringify(settings));
  return settings;
}

export async function getMeetingState(): Promise<MeetingState> {
  if (desktop) return invoke<MeetingState>('get_meeting_state');
  return { status: 'idle', meetingId: null, title: '', startedAt: null, transcript: [], error: null };
}

export type AudioDevice = { direction: 'render' | 'microphone'; label: string; id: string; isDefault: boolean };
export async function audioDevices(): Promise<AudioDevice[]> {
  return desktop ? invoke<AudioDevice[]>('audio_devices') : [];
}

export async function startMeeting(title: string, consent: boolean, selectedMicId: string | null, remoteConsent = false): Promise<MeetingState> {
  if (!desktop) throw new Error('Meeting capture is unavailable in the browser preview.');
  return invoke<MeetingState>('start_meeting', { title, consent, selectedMicId, remoteConsent });
}

export async function stopMeeting(): Promise<MeetingState> {
  if (!desktop) throw new Error('Meeting capture is unavailable in the browser preview.');
  return invoke<MeetingState>('stop_meeting');
}

export async function hideOverlay(): Promise<void> {
  if (desktop) await getCurrentWindow().hide();
}

export type ApiProvider = 'gemini' | 'deepseek';
export type ProviderConfig = { provider: ApiProvider; baseUrl: string; model: string | null; allowFallback: boolean };
export type ModelInfo = { id: string; name: string };
export type AccountMetadata = { accountId: string; displayName: string | null; email: string | null; clientId: string; selectedModel: string | null; signedIn: boolean };
export type ConnectionState = { providers: { config: ProviderConfig; keyConfigured: boolean }[]; accounts: AccountMetadata[]; activeAccount: string | null; signingIn: boolean };
export async function loadConnections(): Promise<ConnectionState> {
  if (desktop) return invoke('get_connections');
  return { providers: [
    { config: { provider: 'gemini', baseUrl: 'https://generativelanguage.googleapis.com/v1beta/openai/', model: null, allowFallback: false }, keyConfigured: false },
    { config: { provider: 'deepseek', baseUrl: 'https://api.deepseek.com/', model: null, allowFallback: false }, keyConfigured: false },
  ], accounts: [], activeAccount: null, signingIn: false };
}
export const saveProviders = (configs: ProviderConfig[]) => invoke<void>('save_providers', { configs });
export const setApiKey = (provider: ApiProvider, key: string) => invoke<void>('set_api_key', { provider, key });
export const deleteApiKey = (provider: ApiProvider) => invoke<void>('delete_api_key', { provider });
export const providerModels = (provider: ApiProvider) => invoke<ModelInfo[]>('provider_models', { provider });
export const beginSignIn = () => invoke<void>('begin_chatgpt_sign_in');
export const cancelSignIn = () => invoke<void>('cancel_chatgpt_sign_in');
export const selectAccount = (accountId: string) => invoke<void>('select_chatgpt_account', { accountId });

export type SavedMeeting = { id: number; title: string; startedAt: number; endedAt: number | null; status: 'completed' | 'interrupted' };
export type TranscriptCursor = { startMs: number; rowId: number };
export type SavedTranscriptPage = { segments: TranscriptLine[]; next: TranscriptCursor | null };
export type SavedMeetingsPage = { meetings: SavedMeeting[]; hasMore: boolean };
export async function savedMeetings(beforeId: number | null = null): Promise<SavedMeetingsPage> {
  return desktop ? invoke<SavedMeetingsPage>('saved_meetings', { beforeId }) : { meetings: [], hasMore: false };
}
export async function savedTranscript(meetingId: number, cursor: TranscriptCursor | null = null): Promise<SavedTranscriptPage> {
  return desktop ? invoke<SavedTranscriptPage>('saved_transcript', { meetingId, cursor }) : { segments: [], next: null };
}
export async function searchTranscript(meetingId: number, query: string): Promise<TranscriptLine[]> {
  return desktop ? invoke<TranscriptLine[]>('search_transcript', { meetingId, query }) : [];
}

export const chatgptModels = (accountId: string) => invoke<ModelInfo[]>('chatgpt_models', { accountId });
export const selectChatgptModel = (accountId: string, modelId: string) => invoke<void>('select_chatgpt_model', { accountId, modelId });
export const reauthorizeAccount = (accountId: string) => invoke<void>('reauthorize_chatgpt_account', { accountId });
export const signOutAccount = (accountId: string) => invoke<boolean>('sign_out_chatgpt_account', { accountId });

export type ResponseState = {
  requestId: number; attemptId: number; meetingId: number;
  status: 'preparing' | 'streaming' | 'completed' | 'partial' | 'error' | 'cancelled';
  answer: string; provider: string | null; model: string | null;
  contextOmitted: boolean; error: string | null;
  usage: { inputTokens: number | null; outputTokens: number | null; totalTokens: number | null } | null;
  userText: string | null;
};
export const getResponseState = (): Promise<ResponseState | null> => desktop ? invoke('get_response_state') : Promise.resolve(null);
export const askMeeting = (meetingId: number, text: string, includeMicrophone: boolean): Promise<ResponseState | null> => invoke('ask_meeting', { meetingId, text, includeMicrophone });
export const sendMeetingSpeech = (meetingId: number): Promise<ResponseState | null> => invoke('send_meeting_speech', { meetingId });
export const cancelResponse = (): Promise<void> => invoke('cancel_response');

export const openExternal = (url: string): Promise<void> => desktop ? invoke('open_external', { url }) : Promise.resolve().then(() => { window.open(url, '_blank', 'noopener,noreferrer'); });

export const restoreSavedMeeting = (meetingId: number): Promise<MeetingState> => invoke('restore_saved_meeting', { meetingId });
