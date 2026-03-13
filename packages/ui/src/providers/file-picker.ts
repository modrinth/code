import { createContext } from '.'

export interface PickedFile {
	/** Browser File object */
	file: File
	/** Native file system path (available on Tauri, undefined on web) */
	path?: string
	/** URL suitable for display (blob URL on web, convertFileSrc URL on Tauri) */
	previewUrl: string
}

export interface FilePickerProvider {
	/** Pick an image file (for icons) */
	pickImage: () => Promise<PickedFile | null>
	/** Pick a .mrpack modpack file */
	pickModpackFile: () => Promise<PickedFile | null>
}

export const [injectFilePicker, provideFilePicker] = createContext<FilePickerProvider>('FilePicker')
