import { invoke, isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

export type Settings = {
  sendMode: 'on_hotkey' | 'automatic';
  responseMode: 'suggested_answers' | 'summary' | 'custom';
  customInstruction: string;
  overlayShortcut: string;
  sendShortcut: string;
  newChatShortcut: string;
  launchOnLogin: boolean;
  includeMicrophone: boolean;
  autoSendDelayMs: number;
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
  overlayShortcut: 'Ctrl+Space', sendShortcut: 'Ctrl+Shift+Enter', newChatShortcut: 'Ctrl+Alt+N', launchOnLogin: false,
  includeMicrophone: true, autoSendDelayMs: 2000,
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
export type AccountMetadata = { accountId: string; displayName: string | null; email: string | null; clientId: string; selectedModel: string | null; signedIn: boolean; planUsageEnabled?: boolean };
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


export type MemoryEntry = { id: number; title: string; body: string; category: string; project: string | null; enabled: boolean; createdAt: number; updatedAt: number; source: 'manual' };
export type MemoryInput = Pick<MemoryEntry, 'title' | 'body' | 'category' | 'project' | 'enabled'>;
export type MemoryPage = { entries: MemoryEntry[]; hasMore: boolean; next: number | null };
export const listMemories = (beforeId: number | null = null): Promise<MemoryPage> => desktop ? invoke('list_memories', { beforeId }) : Promise.resolve({ entries: [], hasMore: false, next: null });
export const searchMemories = (query: string): Promise<MemoryEntry[]> => desktop ? invoke('search_memories', { query }) : Promise.resolve([]);
export const saveMemory = (id: number | null, input: MemoryInput): Promise<MemoryEntry> => invoke('save_memory', { id, input });
export const deleteMemory = (id: number): Promise<void> => invoke('delete_memory', { id });
export async function openMemory(): Promise<void> {
  if (desktop) await invoke('open_memory');
  else window.open('?view=memory', 'harness-memory');
}

export const closeMemory = (): Promise<void> => invoke('close_memory');
export const startChat = (): Promise<MeetingState> => invoke('start_chat');
export const startDragging = (): Promise<void> => desktop ? getCurrentWindow().startDragging() : Promise.resolve();

export type DocumentEntry = { id: number; title: string; sourcePath: string; sourcePolicy: 'reference' | 'copy'; contentHash: string; modifiedAt: number; indexedAt: number; indexingVersion: number; project: string | null; enabled: boolean; chunkCount: number; textBytes: number };
export type DocumentPage = { documents: DocumentEntry[]; hasMore: boolean; next: number | null };
export type DocumentChunk = { id: number; documentId: number; title: string; sourcePath: string; contentHash: string; indexingVersion: number; chunkIndex: number; text: string };
export const listDocuments = (beforeId: number | null = null): Promise<DocumentPage> => desktop ? invoke('list_documents', { beforeId }) : Promise.resolve({ documents: [], hasMore: false, next: null });
export const importDocument = (sourcePolicy: 'reference' | 'copy', project: string | null, enabled: boolean): Promise<DocumentEntry | null> => invoke('import_document', { sourcePolicy, project, enabled });
export const reindexDocument = (id: number): Promise<DocumentEntry> => invoke('reindex_document', { id });
export const setDocumentEnabled = (id: number, enabled: boolean): Promise<DocumentEntry> => invoke('set_document_enabled', { id, enabled });
export const deleteDocument = (id: number): Promise<{ cleanupPending: boolean }> => invoke('delete_document', { id });
export const checkDocument = (id: number): Promise<{ changed: boolean; missing: boolean }> => invoke('check_document', { id });
export const searchDocuments = (query: string): Promise<DocumentChunk[]> => desktop ? invoke('search_documents', { query }) : Promise.resolve([]);
export const closeDocuments = (): Promise<void> => invoke('close_documents');
export async function openDocuments(): Promise<void> { if (desktop) await invoke('open_documents'); else window.open('?view=documents', 'harness-documents'); }

export type ContextSnapshot = { credentialId?:string|null; accountId?:string|null;
  id: number; requestId: number; provider: string; model: string; createdAt: number; upstreamOmitted: boolean;
  request: { model: string; instructions: string | null; messages: { role: string; content: string }[] };
  metadata: { model: string; inputTokenBudget: number; responseReserveTokens: number; estimatedInputTokens: number;
    omissions: { kind: string; count: number }[]; truncations: { kind: string; id?: string; source?: string; startMs?: number; role?: string }[]; excludedIds: string[] };
};
export const requestContext = (requestId: number, beforeId: number | null = null): Promise<ContextSnapshot | null> => desktop ? invoke('request_context', { requestId, beforeId }) : Promise.resolve(null);

export type UsageTotals = {estimatedCostMicros?:number|null;unknownCostAttempts?:number;attempts:number; inputTokens:number|null; outputTokens:number|null; cachedTokens:number|null; unknownTokenAttempts:number; limitHits:number; inFlight:number};
export type UsageGroup = {credentialId?:string|null;provider:string; accountId:string|null; today:UsageTotals; week:UsageTotals};
export type UsageAttempt = {credentialId?:string|null;costMicros?:number|null;id:number; requestId:number; meetingId:number; provider:string; accountId:string|null; model:string; kind:string; status:string; createdAt:number; inputTokens:number|null; outputTokens:number|null; cachedTokens:number|null; firstTokenMs:number|null; totalMs:number|null; errorCode:string|null; limitHit:boolean};
export type UsageReport = {asOf:number; todayStart:number; weekStart:number; groups:UsageGroup[]; groupsTruncated:boolean; attempts:UsageAttempt[]; next:number|null};
export const usageReport = (beforeId:number|null=null):Promise<UsageReport> => desktop ? invoke('usage_report',{beforeId}) : Promise.resolve({asOf:Math.floor(Date.now()/1000),todayStart:0,weekStart:0,groups:[],groupsTruncated:false,attempts:[],next:null});

export type ApiPrice = {inputMicrosPerMillion:number;outputMicrosPerMillion:number;cachedMicrosPerMillion:number|null};
export type ApiBudgetInput = {provider:string;keyId:string;model:string|null;tokenCap:number|null;costCapMicros:number|null;price:ApiPrice|null};
export type ApiBudgetStatus = {tokenCap:number|null;costCapMicros:number|null;price:ApiPrice|null;todayTokens:number;todayCostMicros:number|null;attempts:number;unknownTokens:number;unknownCosts:number;pending:number;reason:string|null;resetAt:number};
export type ApiBudgetProvider = {provider:'gemini'|'deepseek';model:string|null;keyId:string|null;budget:ApiBudgetStatus|null};
export const apiBudgets = ():Promise<ApiBudgetProvider[]> => desktop ? invoke('api_budgets') : Promise.resolve([]);
export const saveApiBudget = (input:ApiBudgetInput):Promise<void> => desktop ? invoke('save_api_budget',{input}) : Promise.reject(new Error('API budgets require the desktop app'));
