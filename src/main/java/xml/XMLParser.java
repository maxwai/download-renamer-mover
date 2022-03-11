package xml;

import java.io.File;
import java.io.IOException;
import java.io.PrintWriter;
import java.util.HashMap;
import java.util.Map;
import java.util.Map.Entry;
import javax.annotation.Nonnull;
import javax.xml.parsers.DocumentBuilderFactory;
import javax.xml.parsers.ParserConfigurationException;
import javax.xml.transform.OutputKeys;
import javax.xml.transform.Transformer;
import javax.xml.transform.TransformerException;
import javax.xml.transform.TransformerFactory;
import javax.xml.transform.dom.DOMSource;
import javax.xml.transform.stream.StreamResult;
import javax.xml.xpath.XPath;
import javax.xml.xpath.XPathConstants;
import javax.xml.xpath.XPathExpressionException;
import javax.xml.xpath.XPathFactory;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.w3c.dom.Attr;
import org.w3c.dom.Document;
import org.w3c.dom.Element;
import org.w3c.dom.Node;
import org.w3c.dom.NodeList;
import org.xml.sax.SAXException;

public class XMLParser {
	
	private static final Logger logger = LoggerFactory.getLogger("XMLParser");
	
	private static final String CONFIG_FILE_NAME = "appdata/Config.xml";
	
	private static final String BOT_TOKEN_TAG = "BotToken";
	private static final String MAIN_CHANNEL_TAG = "MainChannel";
	
	// Mappings
	private static final String MAPPINGS_TAG = "Mappings";
	private static final String MAPPING_SINGLE_TAG = "Mapping";
	private static final String ALTERNATIVE_ATTRIBUTE_TAG = "alternative";
	// Mappings
	
	private static void saveDummyDocument(File file) {
		try {
			PrintWriter writer = new PrintWriter(file);
			writer.write("""
					<?xml version="1.0" encoding="UTF-8" standalone="no"?>
					<root>
					  <BotToken><!--Put here your Bot Token--></BotToken>
					  <MainChannel><!--Put here the Channel ID of the Main Channel--></MainChannel>
					</root>""");
			writer.close();
		} catch (IOException e) {
			e.printStackTrace();
			System.exit(1);
		}
	}
	
	/**
	 * Will write the new XML file to Config.xml
	 *
	 * @param doc The document
	 */
	private static void writeDocument(Document doc) {
		try {
			// remove all '\n' and ' '
			XPathFactory xfact = XPathFactory.newInstance();
			XPath xpath = xfact.newXPath();
			NodeList empty = (NodeList) xpath.evaluate("//text()[normalize-space(.) = '']",
					doc, XPathConstants.NODESET);
			for (int i = 0; i < empty.getLength(); i++) {
				Node node = empty.item(i);
				node.getParentNode().removeChild(node);
			}
			
			// pretty print the xml
			TransformerFactory transformerFactory = TransformerFactory.newInstance();
			Transformer transformer = transformerFactory.newTransformer();
			transformer.setOutputProperty(OutputKeys.METHOD, "xml");
			transformer.setOutputProperty(OutputKeys.INDENT, "yes");
			transformer.setOutputProperty("{http://xml.apache.org/xslt}indent-amount", "2");
			
			// save the xml
			DOMSource source = new DOMSource(doc);
			StreamResult result = new StreamResult(new File(CONFIG_FILE_NAME));
			transformer.transform(source, result);
			logger.info("Saved the Config.xml");
		} catch (TransformerException | XPathExpressionException e) {
			logger.error(
					"Could not save correctly the XML File. See stacktrace for more information");
			e.printStackTrace();
		}
	}
	
	/**
	 * Will get the Config.xml or, if not present, create a dummy one and exit
	 *
	 * @return The Document
	 */
	private static Document getDocument() {
		try {
			File inputFile = new File(CONFIG_FILE_NAME);
			if (inputFile.createNewFile()) {
				saveDummyDocument(inputFile);
				logger.error("There was no " + CONFIG_FILE_NAME
							 + " available. Created a dummy one. Please fill it out");
				System.exit(1);
			}
			return DocumentBuilderFactory.newInstance().newDocumentBuilder().parse(inputFile);
		} catch (ParserConfigurationException | IOException e) {
			e.printStackTrace();
			System.exit(1);
		} catch (SAXException e) {
			xmlFormatException("something went wrong while parsing the xml");
		}
		return null; // will never get there
	}
	
	/**
	 * Will retrieve the Discord Bot Token
	 *
	 * @return The Token
	 */
	@Nonnull
	public static String getBotToken() {
		NodeList nList = getDocument().getElementsByTagName(BOT_TOKEN_TAG);
		if (nList.getLength() == 1) {
			logger.info("Getting the Bot Token");
			return readTextElement(nList.item(0));
		} else
			xmlFormatException("multiple or no Bot Token Tags");
		//noinspection ConstantConditions
		return null; // will never go there
	}
	
	/**
	 * Will retrieve the Main Channel ID
	 *
	 * @return the main Channel ID
	 */
	public static long getMainChannel() {
		NodeList nList = getDocument().getElementsByTagName(MAIN_CHANNEL_TAG);
		if (nList.getLength() == 1) {
			logger.info("Getting the Main Channel ID");
			return Long.parseLong(readTextElement(nList.item(0)));
		} else
			xmlFormatException("multiple or no Main Channel ID Tags");
		return -1; // will never go there
	}
	
	/**
	 * Will get known Mappings if there are any
	 *
	 * @return The Mappings (alt -> OG)
	 */
	public static Map<String, String> getMappings() {
		NodeList nList = getDocument().getElementsByTagName(MAPPINGS_TAG);
		Map<String, String> output = new HashMap<>();
		if (nList.getLength() == 1) {
			try {
				Element element = (Element) nList.item(0);
				NodeList countdownList = element.getElementsByTagName(MAPPING_SINGLE_TAG);
				for (int i = 0; i < countdownList.getLength(); i++) {
					Element countdown = (Element) countdownList.item(i);
					output.put(countdown.getAttribute(ALTERNATIVE_ATTRIBUTE_TAG),
							readTextElement(countdown));
				}
				logger.info("Got the saved Mappings");
			} catch (NullPointerException e) {
				xmlFormatException("Tag missing in Mapping");
			}
		} else
			logger.info("No mappings known");
		return output;
	}
	
	/**
	 * Will add a Mapping to the Mappings
	 *
	 * @param mapping The Mapping to save
	 */
	public static void addMapping(Entry<String, String> mapping) {
		Document doc = getDocument();
		NodeList countdownsNodeList = doc.getElementsByTagName(MAPPINGS_TAG);
		Node mappingsNode;
		if (countdownsNodeList.getLength() == 0) {
			mappingsNode = doc.createElement(MAPPINGS_TAG);
			doc.getFirstChild().appendChild(mappingsNode);
		} else
			mappingsNode = countdownsNodeList.item(0);
		
		Element mappingNode = doc.createElement(MAPPING_SINGLE_TAG);
		
		mappingNode.appendChild(doc.createTextNode(mapping.getValue()));
		Attr attribute = doc.createAttribute(ALTERNATIVE_ATTRIBUTE_TAG);
		attribute.setValue(mapping.getKey());
		mappingNode.setAttributeNode(attribute);
		
		mappingsNode.appendChild(mappingNode);
		
		logger.info("Adding a Mapping");
		writeDocument(doc);
	}
	
	/**
	 * Will trim all '\n' and ' ' at the beginning and end of the Text Element
	 *
	 * @param node The Node were the Text Element is
	 *
	 * @return A String striped of it's unnecessary '\n' and ' '
	 */
	private static String readTextElement(@Nonnull Node node) {
		String text = node.getTextContent();
		if (text == null || text.equals(""))
			return "";
		while (text.charAt(0) == '\n' || text.charAt(0) == ' ') {
			text = text.substring(1);
		}
		while (text.charAt(text.length() - 1) == '\n' || text.charAt(text.length() - 1) == ' ') {
			text = text.substring(0, text.length() - 1);
		}
		return text;
	}
	
	/**
	 * Will output a Error Log and throw a Runtime Exception
	 *
	 * @param reason The Message that should be in the Log
	 */
	private static void xmlFormatException(@Nonnull String reason) {
		logger.error("XML was wrongly formatted: " + reason);
		throw new RuntimeException("XML was wrongly formatted: " + reason);
	}
	
}
