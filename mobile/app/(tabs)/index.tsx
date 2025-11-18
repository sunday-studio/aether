import { ThemedText } from '@/components/ThemedText';
import { parseActivityFromText } from '@/schema/activity.schema';
import { ExpoSpeechRecognitionModule, useSpeechRecognitionEvent } from 'expo-speech-recognition';
import { useState } from 'react';
import { Button, ScrollView, View } from 'react-native';

export default function App() {
  const [recognizing, setRecognizing] = useState(false);
  const [transcript, setTranscript] = useState('');

  useSpeechRecognitionEvent('start', () => setRecognizing(true));
  useSpeechRecognitionEvent('end', () => setRecognizing(false));
  useSpeechRecognitionEvent('result', (event) => {
    setTranscript(event.results[0]?.transcript);
  });
  useSpeechRecognitionEvent('error', (event) => {
    console.log('error code:', event.error, 'error message:', event.message);
  });

  const handleStart = async () => {
    setTranscript('');
    const result = await ExpoSpeechRecognitionModule.requestPermissionsAsync();
    if (!result.granted) {
      console.warn('Permissions not granted', result);
      return;
    }
    // Start speech recognition
    ExpoSpeechRecognitionModule.start({
      lang: 'en-US',
      interimResults: true,
      continuous: false,
    });
  };

  const handleStop = async () => {
    console.log('stopping recognition');
    ExpoSpeechRecognitionModule.stop();

    const activity = await parseActivityFromText(transcript);
    console.log(activity);
  };

  return (
    <View
      style={{
        flex: 1,
        justifyContent: 'center',
        alignItems: 'center',
        backgroundColor: 'white',
        borderWidth: 1,
        borderColor: 'red',
        paddingTop: 400,
      }}
    >
      {!recognizing ? (
        <Button title="Start" onPress={handleStart} />
      ) : (
        <Button title="Stop" onPress={handleStop} />
      )}

      <ScrollView
        contentContainerStyle={{
          alignItems: 'center',
          marginTop: 20,
          borderWidth: 1,
          borderColor: 'blue',
        }}
      >
        <ThemedText style={{ textAlign: 'center' }}>{transcript}</ThemedText>
      </ScrollView>
    </View>
  );
}
