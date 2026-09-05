import { invoke } from "./ipc";

export interface PromptTemplateDto {
  key: string;
  content: string;
  userModified: boolean;
}

/// 获取全部提示词模板。
export const getPromptTemplates = (): Promise<PromptTemplateDto[]> =>
  invoke("get_prompt_templates");

/// 保存单个提示词模板。
export const savePromptTemplate = (key: string, content: string): Promise<PromptTemplateDto> =>
  invoke("save_prompt_template", { input: { key, content } });

/// 重置提示词模板（key 为空则全部重置），返回重置后的全部模板。
export const resetPromptTemplate = (key?: string): Promise<PromptTemplateDto[]> =>
  invoke("reset_prompt_template", { input: { key: key ?? null } });
