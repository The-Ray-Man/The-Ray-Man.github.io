import { ActionIcon, Center, Flex, SimpleGrid, Text } from "@mantine/core";
import { IconBrandGithub, IconHeart } from "@tabler/icons-react";

const Footer = () => {
  return (
    <SimpleGrid cols={3} h={"50"}>
      <div></div>
      <Center>
        <Text pe={"xs"}>Made with ❤️ by TecTrixer and The-Ray-Man</Text>
      </Center>
      <Flex justify={"end"} align={"center"}>
        <a href="https://github.com/The-Ray-Man/Typers">
          <ActionIcon>
            <IconBrandGithub size={24} />
          </ActionIcon>
        </a>
      </Flex>
    </SimpleGrid>
  );
};
export default Footer;
