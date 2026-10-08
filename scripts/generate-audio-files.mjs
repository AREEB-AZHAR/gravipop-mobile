import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');
const assetsAudioDir = path.join(rootDir, 'assets', 'audio');
const publicAudioDir = path.join(rootDir, 'public', 'assets', 'audio');

fs.mkdirSync(assetsAudioDir, { recursive: true });
fs.mkdirSync(publicAudioDir, { recursive: true });

function createWavBuffer(samples, sampleRate = 44100) {
  const numChannels = 1;
  const bitsPerSample = 16;
  const byteRate = sampleRate * numChannels * (bitsPerSample / 8);
  const blockAlign = numChannels * (bitsPerSample / 8);
  const dataLen = samples.length * 2;
  const riffChunkSize = 36 + dataLen;

  const buffer = Buffer.alloc(44 + dataLen);
  buffer.write('RIFF', 0);
  buffer.writeUInt32LE(riffChunkSize, 4);
  buffer.write('WAVE', 8);
  buffer.write('fmt ', 12);
  buffer.writeUInt32LE(16, 16);
  buffer.writeUInt16LE(1, 20); // PCM
  buffer.writeUInt16LE(numChannels, 22);
  buffer.writeUInt32LE(sampleRate, 24);
  buffer.writeUInt32LE(byteRate, 28);
  buffer.writeUInt16LE(blockAlign, 32);
  buffer.writeUInt16LE(bitsPerSample, 34);
  buffer.write('data', 36);
  buffer.writeUInt32LE(dataLen, 40);

  for (let i = 0; i < samples.length; i++) {
    const s = Math.max(-1, Math.min(1, samples[i]));
    const intSample = Math.round(s * 32767);
    buffer.writeInt16LE(intSample, 44 + i * 2);
  }

  return buffer;
}

function generateBellChime(freq, durationSecs = 0.35, sampleRate = 44100) {
  const totalSamples = Math.floor(sampleRate * durationSecs);
  const samples = new Float32Array(totalSamples);
  for (let i = 0; i < totalSamples; i++) {
    const t = i / sampleRate;
    const env = Math.exp(-t * 9.0);
    const fundamental = Math.sin(t * freq * 2.0 * Math.PI);
    const overtone = Math.sin(t * freq * 2.76 * 2.0 * Math.PI) * 0.35;
    samples[i] = (fundamental + overtone) * env * 0.7;
  }
  return createWavBuffer(samples, sampleRate);
}

function generateWoosh(durationSecs = 0.16, sampleRate = 44100) {
  const totalSamples = Math.floor(sampleRate * durationSecs);
  const samples = new Float32Array(totalSamples);
  for (let i = 0; i < totalSamples; i++) {
    const t = i / sampleRate;
    const env = Math.exp(-Math.pow(t - 0.04, 2) * 160.0);
    const pitch = 220.0 + t * 450.0;
    samples[i] = Math.sin(t * pitch * 2.0 * Math.PI) * env * 0.5;
  }
  return createWavBuffer(samples, sampleRate);
}

function generateLowBoom(durationSecs = 0.45, sampleRate = 44100) {
  const totalSamples = Math.floor(sampleRate * durationSecs);
  const samples = new Float32Array(totalSamples);
  for (let i = 0; i < totalSamples; i++) {
    const t = i / sampleRate;
    const env = Math.exp(-t * 6.5);
    const pitch = Math.max(35.0, 140.0 - t * 100.0);
    samples[i] = Math.sin(t * pitch * 2.0 * Math.PI) * env * 0.8;
  }
  return createWavBuffer(samples, sampleRate);
}

function generateClick(durationSecs = 0.035, sampleRate = 44100) {
  const totalSamples = Math.floor(sampleRate * durationSecs);
  const samples = new Float32Array(totalSamples);
  for (let i = 0; i < totalSamples; i++) {
    const t = i / sampleRate;
    const env = Math.exp(-t * 140.0);
    samples[i] = Math.sin(t * 880.0 * 2.0 * Math.PI) * env * 0.45;
  }
  return createWavBuffer(samples, sampleRate);
}

const freqs = [523.25, 587.33, 659.25, 783.99, 880.00, 1046.50, 1174.66, 1318.51];
const files = [];

freqs.forEach((freq, idx) => {
  const filename = `chime_${idx}.wav`;
  const buf = generateBellChime(freq);
  files.push({ name: filename, buffer: buf });
});

files.push({ name: 'slingshot.wav', buffer: generateWoosh() });
files.push({ name: 'game_over.wav', buffer: generateLowBoom() });
files.push({ name: 'click.wav', buffer: generateClick() });

for (const { name, buffer } of files) {
  const dest1 = path.join(assetsAudioDir, name);
  const dest2 = path.join(publicAudioDir, name);
  fs.writeFileSync(dest1, buffer);
  fs.writeFileSync(dest2, buffer);
  console.log(`Generated: ${name} (${buffer.length} bytes)`);
}

console.log('Successfully generated and saved all static audio WAV files.');
