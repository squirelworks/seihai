// Headless salvage export: functions, imports/exports, interesting strings,
// decompiled C for key functions, and machine-readable summary stats.
// Args: <outputDir> [maxDecompFuncs]
//@category Salvage
//@runtime Java

import java.io.*;
import java.nio.charset.StandardCharsets;
import java.util.*;
import java.util.regex.Pattern;

import ghidra.app.decompiler.*;
import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.address.AddressIterator;
import ghidra.program.model.listing.*;
import ghidra.program.model.symbol.*;

public class ExportSalvageArtifacts extends GhidraScript {

	private static final int DEFAULT_MAX_DECOMP = 40;
	private static final int MIN_STRING_LEN = 4;
	private static final int MAX_STRING_LEN = 512;

	private static final Pattern INTERESTING = Pattern.compile(
		"(?i)(/[\\w./~-]{3,}|\\\\[\\w.\\\\-]{3,}|" +
			"https?://|ftp://|\\w+://|" +
			"--?[a-zA-Z][\\w-]{0,32}|" +
			"\\.(conf|cfg|ini|json|xml|yaml|yml|toml|properties|log|db|sqlite)|" +
			"usage|help|version|config|connect|listen|bind|socket|port|" +
			"password|passwd|token|secret|api[_-]?key|auth|" +
			"error|fail|warn|debug|fatal|" +
			"SELECT |INSERT |UPDATE |DELETE |FROM |" +
			"main|argv|getenv|fopen|open\\(|printf|sprintf|" +
			"grpc|protobuf|http|tcp|udp|unix|pipe|" +
			"gserver|graal|seihai|npc|player)");

	private static final Pattern KEY_FUNC_NAME = Pattern.compile(
		"(?i)^(main|_start|start|WinMain|wmain|usage|help|print_?usage|" +
			"parse_?args?|getopt|init|setup|config|load_?config|read_?config|" +
			"connect|listen|bind|accept|serve|run|loop|dispatch|" +
			"handle_.*|process_.*|cmd_.*|do_.*|" +
			"encrypt|decrypt|auth|login|verify|" +
			"StartServer.*|SavePlayerCount.*|RequestServer.*|GetServer.*|" +
			"parse(__|Login__).*|SendAccountInfo.*|CheckActivatePlayer.*|" +
			"CheckPlayerLimit.*|CheckCorrectIP.*|dbthread_main.*).*$");

	// Static binaries embed libc; deprioritize those when app symbols exist.
	private static final Pattern LIBC_NOISE = Pattern.compile(
		"(?i)^(__|_IO_|_dl_|_nl_|_nss_|_res_|_pthread_|_mcount|" +
			"strto|v?f?printf|v?f?scanf|inflate_|deflate_|malloc|free|" +
			"memcpy|memmove|memset|strlen|strcpy|strcmp|sprintf|snprintf|" +
			"gettext|dcigettext|setlocale|tzfile|md5_|crypt|gconv_|mpn_|" +
			"xdr_|svcauth|getmntent|cleanup_dirname).*");

	@Override
	public void run() throws Exception {
		if (currentProgram == null) {
			printerr("No current program");
			return;
		}

		String[] args = getScriptArgs();
		if (args == null || args.length < 1) {
			printerr("Usage: ExportSalvageArtifacts.java <outputDir> [maxDecompFuncs]");
			return;
		}

		File outDir = new File(args[0]);
		if (!outDir.exists() && !outDir.mkdirs()) {
			printerr("Cannot create output dir: " + outDir);
			return;
		}

		int maxDecomp = DEFAULT_MAX_DECOMP;
		if (args.length >= 2) {
			try {
				maxDecomp = Integer.parseInt(args[1]);
			}
			catch (NumberFormatException e) {
				printerr("Invalid maxDecompFuncs, using " + DEFAULT_MAX_DECOMP);
			}
		}

		File decompDir = new File(outDir, "decompiled");
		decompDir.mkdirs();

		println("Exporting salvage artifacts for " + currentProgram.getName() + " -> " + outDir);

		List<FuncRec> functions = collectFunctions();
		writeFunctions(new File(outDir, "functions.txt"), functions);
		writeFunctionsCsv(new File(outDir, "functions.csv"), functions);

		ImpExp ie = collectImportsExports();
		writeLines(new File(outDir, "imports.txt"), ie.imports);
		writeLines(new File(outDir, "exports.txt"), ie.exports);

		List<String> interesting = collectInterestingStrings();
		writeLines(new File(outDir, "strings_interesting.txt"), interesting);

		List<FuncRec> keyFuncs = selectKeyFunctions(functions, maxDecomp);
		int decompiled = decompileKeyFunctions(keyFuncs, decompDir);

		writeSummaryStats(new File(outDir, "summary_stats.txt"), functions, ie, interesting,
			decompiled, keyFuncs);

		println("Export complete: " + functions.size() + " funcs, " + ie.imports.size() +
			" imports, " + ie.exports.size() + " exports, " + interesting.size() +
			" interesting strings, " + decompiled + " decompiled");
	}

	private static class FuncRec {
		String name;
		String entry;
		long size;
		int paramCount;
		boolean external;
		boolean thunk;
		String signature;
		int callers;
		int callees;
		boolean entryPoint;
	}

	private static class ImpExp {
		List<String> imports = new ArrayList<>();
		List<String> exports = new ArrayList<>();
	}

	private List<FuncRec> collectFunctions() throws Exception {
		List<FuncRec> out = new ArrayList<>();
		FunctionManager fm = currentProgram.getFunctionManager();
		Set<String> entryPoints = new HashSet<>();
		AddressIterator entries =
			currentProgram.getSymbolTable().getExternalEntryPointIterator();
		while (entries.hasNext()) {
			monitor.checkCancelled();
			entryPoints.add(entries.next().toString());
		}

		FunctionIterator it = fm.getFunctions(true);
		while (it.hasNext()) {
			monitor.checkCancelled();
			Function f = it.next();
			FuncRec r = new FuncRec();
			r.name = f.getName(true);
			r.entry = f.getEntryPoint().toString();
			r.size = f.getBody().getNumAddresses();
			r.paramCount = f.getParameterCount();
			r.external = f.isExternal();
			r.thunk = f.isThunk();
			r.signature = f.getPrototypeString(true, false);
			r.callers = safeSize(f.getCallingFunctions(monitor));
			r.callees = safeSize(f.getCalledFunctions(monitor));
			r.entryPoint = entryPoints.contains(r.entry);
			out.add(r);
		}
		out.sort((a, b) -> Long.compare(b.size, a.size));
		return out;
	}

	private ImpExp collectImportsExports() throws Exception {
		ImpExp ie = new ImpExp();
		SymbolTable st = currentProgram.getSymbolTable();

		// Imports: external symbols / EXTERNAL namespace
		for (Symbol s : st.getExternalSymbols()) {
			monitor.checkCancelled();
			String lib = "";
			Symbol parent = null;
			Namespace ns = s.getParentNamespace();
			if (ns != null) {
				lib = ns.getName(true);
			}
			String line = s.getName(true);
			if (!lib.isEmpty() && !lib.equals("EXTERNAL")) {
				line = lib + " :: " + s.getName();
			}
			ie.imports.add(line + " @ " + s.getAddress());
		}
		Collections.sort(ie.imports);

		// Exports / program entry points
		AddressIterator ep = st.getExternalEntryPointIterator();
		while (ep.hasNext()) {
			monitor.checkCancelled();
			Address a = ep.next();
			Symbol s = st.getPrimarySymbol(a);
			String name = s != null ? s.getName(true) : a.toString();
			String kind = (s != null && s.getSymbolType() != null)
				? s.getSymbolType().toString()
				: "entry";
			ie.exports.add(name + " @ " + a + " [" + kind + "]");
		}

		// Global function symbols that look exported (non-default names)
		FunctionIterator fit = currentProgram.getFunctionManager().getFunctions(true);
		while (fit.hasNext()) {
			monitor.checkCancelled();
			Function f = fit.next();
			if (f.isExternal() || f.isThunk()) {
				continue;
			}
			Symbol s = f.getSymbol();
			if (s == null) {
				continue;
			}
			if (s.getSource() == SourceType.IMPORTED || s.getSource() == SourceType.USER_DEFINED ||
				s.getSource() == SourceType.ANALYSIS) {
				String n = f.getName();
				if (n.startsWith("FUN_") || n.startsWith("thunk_")) {
					continue;
				}
				String line = f.getName(true) + " @ " + f.getEntryPoint() + " [function]";
				if (!ie.exports.contains(line)) {
					// Prefer listing meaningful named globals as candidate exports
					if (s.getSource() == SourceType.IMPORTED ||
						(!n.startsWith("entry") && Character.isLetter(n.charAt(0)))) {
						ie.exports.add(line);
					}
				}
			}
		}
		Collections.sort(ie.exports);
		return ie;
	}

	private List<String> collectInterestingStrings() throws Exception {
		List<String> out = new ArrayList<>();
		Listing listing = currentProgram.getListing();
		DataIterator dit = listing.getDefinedData(true);
		Set<String> seen = new LinkedHashSet<>();
		while (dit.hasNext()) {
			monitor.checkCancelled();
			Data d = dit.next();
			if (!d.hasStringValue()) {
				continue;
			}
			String val;
			try {
				Object v = d.getValue();
				val = v == null ? null : v.toString();
			}
			catch (Exception e) {
				continue;
			}
			if (val == null) {
				continue;
			}
			val = sanitizeString(val);
			if (val.length() < MIN_STRING_LEN || val.length() > MAX_STRING_LEN) {
				continue;
			}
			if (!INTERESTING.matcher(val).find()) {
				continue;
			}
			String line = d.getAddress() + "\t" + val;
			if (seen.add(line)) {
				out.add(line);
			}
		}
		return out;
	}

	private List<FuncRec> selectKeyFunctions(List<FuncRec> all, int max) {
		LinkedHashMap<String, FuncRec> picked = new LinkedHashMap<>();

		boolean hasAppSymbols = false;
		for (FuncRec f : all) {
			String s = simpleName(f.name);
			if (!f.external && !f.thunk && !s.startsWith("FUN_") &&
				!LIBC_NOISE.matcher(s).matches()) {
				hasAppSymbols = true;
				break;
			}
		}

		// 1) Name-matched helpers. Do NOT treat every ELF export as an entry —
		//    static binaries export hundreds of libc symbols as "entry points".
		for (FuncRec f : all) {
			if (f.external || f.thunk) {
				continue;
			}
			String simple = simpleName(f.name);
			if (hasAppSymbols && LIBC_NOISE.matcher(simple).matches()) {
				continue;
			}
			boolean isPrimaryEntry = simple.equals("main") || simple.equals("_start") ||
				simple.equals("WinMain") || simple.equals("wmain") ||
				simple.equals("entry") || simple.startsWith("main_");
			if (isPrimaryEntry || KEY_FUNC_NAME.matcher(simple).matches()) {
				picked.put(f.entry, f);
			}
		}

		// 2) Prefer large *application* bodies when symbols exist; else any large body.
		int sizeSlots = Math.max(1, (max - picked.size()) * 3 / 5);
		List<FuncRec> bySize = new ArrayList<>();
		for (FuncRec f : all) {
			if (f.external || f.thunk || picked.containsKey(f.entry) || f.size < 32) {
				continue;
			}
			String s = simpleName(f.name);
			if (hasAppSymbols && LIBC_NOISE.matcher(s).matches()) {
				continue;
			}
			bySize.add(f);
		}
		bySize.sort((a, b) -> Long.compare(b.size, a.size));
		for (FuncRec f : bySize) {
			if (sizeSlots-- <= 0 || picked.size() >= max) {
				break;
			}
			picked.put(f.entry, f);
		}

		// 3) Fill with high connectivity (callers+callees), then remaining size
		List<FuncRec> ranked = new ArrayList<>();
		for (FuncRec f : all) {
			if (f.external || f.thunk || picked.containsKey(f.entry) || f.size < 16) {
				continue;
			}
			String s = simpleName(f.name);
			if (hasAppSymbols && LIBC_NOISE.matcher(s).matches()) {
				continue;
			}
			ranked.add(f);
		}
		ranked.sort((a, b) -> {
			int c = Integer.compare(b.callers + b.callees, a.callers + a.callees);
			if (c != 0) {
				return c;
			}
			return Long.compare(b.size, a.size);
		});
		for (FuncRec f : ranked) {
			if (picked.size() >= max) {
				break;
			}
			picked.put(f.entry, f);
		}

		List<FuncRec> result = new ArrayList<>(picked.values());
		if (result.size() > max) {
			return result.subList(0, max);
		}
		return result;
	}

	private int decompileKeyFunctions(List<FuncRec> keyFuncs, File decompDir) throws Exception {
		DecompInterface ifc = new DecompInterface();
		DecompileOptions options = new DecompileOptions();
		ifc.setOptions(options);
		ifc.toggleCCode(true);
		ifc.toggleSyntaxTree(true);
		ifc.setSimplificationStyle("decompile");
		if (!ifc.openProgram(currentProgram)) {
			printerr("Decompiler failed to open program: " + ifc.getLastMessage());
			ifc.dispose();
			return 0;
		}

		int ok = 0;
		FunctionManager fm = currentProgram.getFunctionManager();
		List<String> index = new ArrayList<>();

		try {
			for (FuncRec rec : keyFuncs) {
				monitor.checkCancelled();
				Function f = fm.getFunctionAt(
					currentProgram.getAddressFactory().getAddress(rec.entry));
				if (f == null) {
					continue;
				}
				monitor.setMessage("Decompiling " + f.getName());
				DecompileResults res = ifc.decompileFunction(f, 60, monitor);
				if (res == null || !res.decompileCompleted()) {
					printerr("Decompile failed: " + f.getName() + " @ " + rec.entry);
					continue;
				}
				String c = res.getDecompiledFunction() != null
					? res.getDecompiledFunction().getC()
					: null;
				if (c == null || c.isBlank()) {
					continue;
				}
				String safe = safeFileName(simpleName(f.getName()) + "_" + rec.entry);
				File out = new File(decompDir, safe + ".c");
				try (Writer w = new OutputStreamWriter(new FileOutputStream(out),
					StandardCharsets.UTF_8)) {
					w.write("/* " + f.getName(true) + " @ " + rec.entry + " size=" + rec.size +
						" */\n");
					w.write(c);
					if (!c.endsWith("\n")) {
						w.write("\n");
					}
				}
				index.add(out.getName() + "\t" + f.getName(true) + "\t" + rec.entry);
				ok++;
			}
		}
		finally {
			ifc.dispose();
		}

		writeLines(new File(decompDir, "INDEX.txt"), index);
		return ok;
	}

	private void writeSummaryStats(File file, List<FuncRec> functions, ImpExp ie,
			List<String> strings, int decompiled, List<FuncRec> keyFuncs) throws IOException {
		List<String> lines = new ArrayList<>();
		lines.add("program=" + currentProgram.getName());
		lines.add("executable_path=" + currentProgram.getExecutablePath());
		lines.add("executable_format=" + currentProgram.getExecutableFormat());
		lines.add("language=" + currentProgram.getLanguageID().getIdAsString());
		lines.add("compiler=" + currentProgram.getCompilerSpec().getCompilerSpecID().getIdAsString());
		lines.add("image_base=" + currentProgram.getImageBase());
		lines.add("function_count=" + functions.size());
		long internal = functions.stream().filter(f -> !f.external && !f.thunk).count();
		lines.add("internal_function_count=" + internal);
		lines.add("import_count=" + ie.imports.size());
		lines.add("export_count=" + ie.exports.size());
		lines.add("interesting_string_count=" + strings.size());
		lines.add("decompiled_count=" + decompiled);
		lines.add("key_function_count=" + keyFuncs.size());

		StringBuilder top = new StringBuilder();
		int n = 0;
		for (FuncRec f : functions) {
			if (f.external || f.thunk) {
				continue;
			}
			if (n++ >= 15) {
				break;
			}
			if (top.length() > 0) {
				top.append(" | ");
			}
			top.append(simpleName(f.name)).append("@").append(f.entry).append("(sz=").append(f.size)
				.append(")");
		}
		lines.add("top_functions=" + top);

		StringBuilder keys = new StringBuilder();
		for (FuncRec f : keyFuncs) {
			if (keys.length() > 0) {
				keys.append(" | ");
			}
			keys.append(simpleName(f.name)).append("@").append(f.entry);
		}
		lines.add("key_functions=" + keys);

		// Sample interesting strings for summary
		StringBuilder ss = new StringBuilder();
		int sc = 0;
		for (String s : strings) {
			if (sc++ >= 25) {
				break;
			}
			String only = s.contains("\t") ? s.substring(s.indexOf('\t') + 1) : s;
			only = only.replace('|', '/');
			if (only.length() > 80) {
				only = only.substring(0, 77) + "...";
			}
			if (ss.length() > 0) {
				ss.append(" | ");
			}
			ss.append(only);
		}
		lines.add("sample_strings=" + ss);

		writeLines(file, lines);
	}

	private void writeFunctions(File file, List<FuncRec> functions) throws IOException {
		List<String> lines = new ArrayList<>();
		lines.add("# name\tentry\tsize\tparams\tcallers\tcallees\texternal\tthunk\tentry_point\tsignature");
		for (FuncRec f : functions) {
			lines.add(String.join("\t", f.name, f.entry, Long.toString(f.size),
				Integer.toString(f.paramCount), Integer.toString(f.callers),
				Integer.toString(f.callees), Boolean.toString(f.external),
				Boolean.toString(f.thunk), Boolean.toString(f.entryPoint),
				f.signature == null ? "" : f.signature.replace('\t', ' ')));
		}
		writeLines(file, lines);
	}

	private void writeFunctionsCsv(File file, List<FuncRec> functions) throws IOException {
		List<String> lines = new ArrayList<>();
		lines.add("name,entry,size,params,callers,callees,external,thunk,entry_point,signature");
		for (FuncRec f : functions) {
			lines.add(csv(f.name) + "," + csv(f.entry) + "," + f.size + "," + f.paramCount + "," +
				f.callers + "," + f.callees + "," + f.external + "," + f.thunk + "," +
				f.entryPoint + "," + csv(f.signature));
		}
		writeLines(file, lines);
	}

	private static void writeLines(File file, List<String> lines) throws IOException {
		try (Writer w = new OutputStreamWriter(new FileOutputStream(file), StandardCharsets.UTF_8)) {
			for (String line : lines) {
				w.write(line);
				w.write("\n");
			}
		}
	}

	private static int safeSize(Set<?> s) {
		return s == null ? 0 : s.size();
	}

	private static String simpleName(String name) {
		if (name == null) {
			return "func";
		}
		int idx = name.lastIndexOf("::");
		if (idx >= 0) {
			return name.substring(idx + 2);
		}
		idx = name.lastIndexOf('/');
		if (idx >= 0) {
			return name.substring(idx + 1);
		}
		return name;
	}

	private static String safeFileName(String s) {
		return s.replaceAll("[^A-Za-z0-9._+-]+", "_");
	}

	private static String sanitizeString(String s) {
		StringBuilder b = new StringBuilder(s.length());
		for (int i = 0; i < s.length(); i++) {
			char c = s.charAt(i);
			if (c == '\n' || c == '\r' || c == '\t') {
				b.append(' ');
			}
			else if (c >= 32 && c < 127) {
				b.append(c);
			}
			else {
				b.append('?');
			}
		}
		return b.toString().trim();
	}

	private static String csv(String s) {
		if (s == null) {
			return "\"\"";
		}
		return "\"" + s.replace("\"", "\"\"") + "\"";
	}
}
