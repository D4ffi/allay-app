
interface Props {
    text: string;
    backgroundColor: string;
    textColor: string;
}

export default function Badges({ text, backgroundColor, textColor }: Props) {
    return (
        <span 
            className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium"
            style={{ 
                backgroundColor: backgroundColor,
                color: textColor 
            }}
        >
            {text}
        </span>
    );
}