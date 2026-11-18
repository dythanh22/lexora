import en from './en';
import vi from './vi';

const dicts: Record<string, Record<string, string>> = { vi, en };

/** Ngôn ngữ giao diện. Đổi `i18n.lang` sẽ cập nhật mọi chỗ gọi `t()` trong template. */
export const i18n = $state({ lang: 'vi' });

export function t(key: string): string {
  return dicts[i18n.lang]?.[key] ?? dicts['vi']?.[key] ?? key;
}
