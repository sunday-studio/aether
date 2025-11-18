// polyfills.js
import { polyfillGlobal } from 'react-native/Libraries/Utilities/PolyfillFunctions';

import { ReadableStream } from 'readable-stream';
polyfillGlobal('ReadableStream', () => ReadableStream); // polyfill ReadableStream

import '@azure/core-asynciterator-polyfill'; // polyfill Async Iterator
import '@bacons/text-decoder/install'; // polyfill Text Decoder
import 'react-native-get-random-values'; // polyfill getRandomValues
