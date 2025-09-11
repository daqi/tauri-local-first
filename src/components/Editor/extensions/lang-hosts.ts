import { StreamLanguage } from '@codemirror/language';
import type { Extension } from '@codemirror/state';

// IPv6 地址由 1-4 位十六进制数字组成的组构成
const hex = '[0-9a-fA-F]{1,4}';
// IPv6 地址可以在最后 4 个字节中嵌入 IPv4 地址
const octet = '(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])';
const ipv4 = `(${octet}\\.){3}${octet}`;
// 前导 0 可以压缩，连续的 0 可以压缩为 "::" ，但是只能出现一次
// 示例：
// 标准 IPv6 "2001:0db8:85a3:0000:0000:8a2e:0370:7334"
// 压缩 IPv6 "2001:db8:85a3::8a2e:370:7334"
// 嵌入 IPv4 "1:2:3:4:5:6:1.2.3.4"
// 压缩 "0:0:0:0:0:0:0:1" 为 "::1"
const ipv6RegexCases = [];
ipv6RegexCases[0] = `((${hex}:){7}(${hex}|:))`; // g:g:g:g:g:g:g:g  g:g:g:g:g:g:g::
ipv6RegexCases[1] = `((${hex}:){6}(:${hex}|${ipv4}|:))`; // g:g:g:g:g:g::g  g:g:g:g:g:g:ip4  g:g:g:g:g:g::
ipv6RegexCases[2] = `((${hex}:){5}(((:${hex}){1,2})|:${ipv4}|:))`; // g:g:g:g:g::g  g:g:g:g:g::g:g  g:g:g:g:g::ip4  g:g:g:g:g::
ipv6RegexCases[3] = `((${hex}:){4}(((:${hex}){1,3})|((:${hex})?:${ipv4})|:))`; // g:g:g:g::g ... g:g:g:g::g:g:g  g:g:g:g::ip4  g:g:g:g::g:ip4  g:g:g:g::
ipv6RegexCases[4] = `((${hex}:){3}(((:${hex}){1,4})|((:${hex}){0,2}:${ipv4})|:))`; // g:g:g::g ... g:g:g::g:g:g:g  g:g:g::ip4 ... g:g:g::g:g:ip4  g:g:g::
ipv6RegexCases[5] = `((${hex}:){2}(((:${hex}){1,5})|((:${hex}){0,3}:${ipv4})|:))`; // g:g::g ... g:g::g:g:g:g:g  g:g::ip4 ... g:g::g:g:g:ip4  g:g::
ipv6RegexCases[6] = `((${hex}:){1}(((:${hex}){1,6})|((:${hex}){0,4}:${ipv4})|:))`; // g::g ... g::g:g:g:g:g:g  g::ip4 ... g::g:g:g:g:ip4  g::
ipv6RegexCases[7] = `(:(((:${hex}){1,7})|((:${hex}){0,5}:${ipv4})|:))`; // ::g ... ::g:g:g:g:g:g:g  ::g:ip4 ... ::g:g:g:g:g:ip4  ::
const ipv6Regex = new RegExp(`^(${ipv6RegexCases.join('|')})\\b`, 'i');

// IPv4
const octet1 = '([01]?[0-9][0-9]?|2[0-4][0-9]|25[0-5]?)'; // 0-255,支持前导零
const ipv4Regex = new RegExp(`^${octet1}(\\.${octet1}){3}\\b`);

/**
 * A simple CodeMirror 6 stream-based mode for hosts files.
 * Highlights:
 * - comments (starting with #)
 * - IPv4/IPv6 addresses
 * - hostnames / domains
 *
 * Usage: include `hosts()` in your editor extensions.
 */

const hostsParser = StreamLanguage.define({
  startState() {
    return {};
  },

  token(stream) {
    // skip spaces
    if (stream.eatSpace()) return null;

    // comment: '#' to end of line
    if (stream.peek() === '#') {
      stream.skipToEnd();
      return 'comment';
    }

    // IPv4
    if ((stream).match(ipv4Regex)) {
      return 'number';
    }
    
    // IPv6
    if (stream.match(ipv6Regex)) {
      return 'number';
    }

    // hostname/domain/token (letters, digits, hyphen, underscore, dots)
    if (stream.match(/^[A-Za-z0-9._-]+/)) {
      return 'atom';
    }

    // punctuation / separators (tabs, colons, etc.)
    stream.next();
    return null;
  },

  languageData: {
    commentTokens: { line: '#' },
    wordChars: /[\w.-]/,
  },
});

/**
 * Combined extension to add to your editor:
 * - language support (tokenizer)
 */
export function hosts(): Extension {
  return [hostsParser];
}

export default hosts;
