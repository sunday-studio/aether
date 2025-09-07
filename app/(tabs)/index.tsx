// // import { Image } from 'expo-image';
// // import { Platform, StyleSheet } from 'react-native';

// // import { HelloWave } from '@/components/HelloWave';
// // import ParallaxScrollView from '@/components/ParallaxScrollView';
// // import { ThemedText } from '@/components/ThemedText';
// // import { ThemedView } from '@/components/ThemedView';

// // export default function HomeScreen() {
// //   return (
// //     <ParallaxScrollView
// //       headerBackgroundColor={{ light: '#A1CEDC', dark: '#1D3D47' }}
// //       headerImage={
// //         <Image
// //           source={require('@/assets/images/partial-react-logo.png')}
// //           style={styles.reactLogo}
// //         />
// //       }>
// //       <ThemedView style={styles.titleContainer}>
// //         <ThemedText type="title">Welcome!</ThemedText>
// //         <HelloWave />
// //       </ThemedView>
// //       <ThemedView style={styles.stepContainer}>
// //         <ThemedText type="subtitle">Step 1: Try it</ThemedText>
// //         <ThemedText>
// //           Edit <ThemedText type="defaultSemiBold">app/(tabs)/index.tsx</ThemedText> to see changes.
// //           Press{' '}
// //           <ThemedText type="defaultSemiBold">
// //             {Platform.select({
// //               ios: 'cmd + d',
// //               android: 'cmd + m',
// //               web: 'F12',
// //             })}
// //           </ThemedText>{' '}
// //           to open developer tools.
// //         </ThemedText>
// //       </ThemedView>
// //       <ThemedView style={styles.stepContainer}>
// //         <ThemedText type="subtitle">Step 2: Explore</ThemedText>
// //         <ThemedText>
// //           {`Tap the Explore tab to learn more about what's included in this starter app.`}
// //         </ThemedText>
// //       </ThemedView>
// //       <ThemedView style={styles.stepContainer}>
// //         <ThemedText type="subtitle">Step 3: Get a fresh start</ThemedText>
// //         <ThemedText>
// //           {`When you're ready, run `}
// //           <ThemedText type="defaultSemiBold">npm run reset-project</ThemedText> to get a fresh{' '}
// //           <ThemedText type="defaultSemiBold">app</ThemedText> directory. This will move the current{' '}
// //           <ThemedText type="defaultSemiBold">app</ThemedText> to{' '}
// //           <ThemedText type="defaultSemiBold">app-example</ThemedText>.
// //         </ThemedText>
// //       </ThemedView>
// //     </ParallaxScrollView>
// //   );
// // }

// // const styles = StyleSheet.create({
// //   titleContainer: {
// //     flexDirection: 'row',
// //     alignItems: 'center',
// //     gap: 8,
// //   },
// //   stepContainer: {
// //     gap: 8,
// //     marginBottom: 8,
// //   },
// //   reactLogo: {
// //     height: 178,
// //     width: 290,
// //     bottom: 0,
// //     left: 0,
// //     position: 'absolute',
// //   },
// // });

import {ExpoSpeechRecognitionModule, useSpeechRecognitionEvent} from "expo-speech-recognition";
import {useState} from "react";
import {Button, ScrollView, Text, View} from "react-native";

export default function App() {
  const [recognizing, setRecognizing] = useState(false);
  const [transcript, setTranscript] = useState("");

  useSpeechRecognitionEvent("start", () => setRecognizing(true));
  useSpeechRecognitionEvent("end", () => setRecognizing(false));
  useSpeechRecognitionEvent("result", (event) => {
    setTranscript(event.results[0]?.transcript);
  });
  useSpeechRecognitionEvent("error", (event) => {
    console.log("error code:", event.error, "error message:", event.message);
  });

  const handleStart = async () => {
    const result = await ExpoSpeechRecognitionModule.requestPermissionsAsync();
    if (!result.granted) {
      console.warn("Permissions not granted", result);
      return;
    }
    // Start speech recognition
    ExpoSpeechRecognitionModule.start({
      lang: "en-US",
      interimResults: true,
      continuous: false,
    });
  };

  return (
    <View>
      {!recognizing ? (
        <Button title="Start" onPress={handleStart} />
      ) : (
        <Button title="Stop" onPress={() => ExpoSpeechRecognitionModule.stop()} />
      )}

      <ScrollView>
        <Text>{transcript}</Text>
      </ScrollView>
    </View>
  );
}

// import {
//   AudioModule,
//   RecordingPresets,
//   setAudioModeAsync,
//   useAudioRecorder,
//   useAudioRecorderState,
// } from "expo-audio";
// import {useEffect} from "react";
// import {Alert, Button, StyleSheet, View} from "react-native";
// import {ExpoSpeechRecognitionModule, useSpeechRecognitionEvent} from "expo-speech-recognition";

// export default function App() {
//   const audioRecorder = useAudioRecorder(RecordingPresets.HIGH_QUALITY);
//   const recorderState = useAudioRecorderState(audioRecorder);

//   const record = async () => {
//     await audioRecorder.prepareToRecordAsync();
//     audioRecorder.record();
//   };

//   const stopRecording = async () => {
//     // The recording will be available on `audioRecorder.uri`.
//     await audioRecorder.stop();

//     console.log("data ->", audioRecorder.uri);
//   };

//   useEffect(() => {
//     (async () => {
//       const status = await AudioModule.requestRecordingPermissionsAsync();
//       if (!status.granted) {
//         Alert.alert("Permission to access microphone was denied");
//       }

//       setAudioModeAsync({
//         playsInSilentMode: true,
//         allowsRecording: true,
//       });
//     })();
//   }, []);

//   return (
//     <View style={styles.container}>
//       <Button
//         title={recorderState.isRecording ? "Stop Recording" : "Start Recording"}
//         onPress={recorderState.isRecording ? stopRecording : record}
//       />
//     </View>
//   );
// }

// const styles = StyleSheet.create({
//   container: {
//     flex: 1,
//     justifyContent: "center",
//     backgroundColor: "#ecf0f1",
//     padding: 10,
//   },
// });
