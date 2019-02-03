const React = require('react');
const CompLibrary = require('../../core/CompLibrary.js');

const Container = CompLibrary.Container;
const GridBlock = CompLibrary.GridBlock;

const siteConfig = require(`${process.cwd()}/siteConfig.js`);

function docUrl(doc, language) {
  return `${siteConfig.baseUrl}docs/${language ? `${language}/` : ''}${doc}`;
}

const Block = props => (
  <Container
    padding={['bottom', 'top']}
    id={props.id}
    background={props.background}>
    <GridBlock align="center" contents={props.children} layout={props.layout} />
  </Container>
);


const Agents = {
  content: 'Agent specific documentation for datastore specific details',
  title: `[Official Agents](${siteConfig.baseUrl}docs/agents)`,
};

const Introduction = {
  content: 'Why agents and what are they for?<br />Documentation common to all agents can be found here',
  title: `[Introduction](${siteConfig.baseUrl}docs/intro)`,
};


class Index extends React.Component {
  render() {
    const language = this.props.language || '';
    return (
      <div>
        <div className="mainContainer">
          <Block key="links" layout="twoColumn">
            {[
              Introduction,
              Agents,
            ]}
          </Block>
        </div>
      </div>
    );
  }
}

module.exports = Index;
