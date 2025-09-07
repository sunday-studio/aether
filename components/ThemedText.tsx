import { Text, type TextProps } from 'react-native';

export type ThemedTextProps = TextProps;

export function ThemedText({ style, ...rest }: ThemedTextProps) {
  return (
    <Text
      style={[
        { fontFamily: 'Inter-Medium', fontSize: 18, lineHeight: 24, color: '#626262' },
        style,
      ]}
      {...rest}
    />
  );
}
