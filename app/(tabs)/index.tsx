import { ThemedText } from '@/components/ThemedText';
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
        <Button title="Stop" onPress={() => ExpoSpeechRecognitionModule.stop()} />
      )}

      <ThemedText style={{ textAlign: 'center' }}>Hello wrold</ThemedText>
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
