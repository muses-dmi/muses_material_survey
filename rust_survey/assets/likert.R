# Exaxmple visualization script for Muses Material Survey Likert data 
# The actual graphs we want to produce will vary, depending on what we want to show 
# for a given presentation of the data.
#
# Benedict R. Gaster

library(ggplot2)
library(dplyr)
library(sqldf)
library(ggpubr)

# Load Likert CSV files, appending all to a single table
likert_files = list.files("/Users/br-gaster/dev/bgaster.github.io/muses_material_survey/rust_survey/assets/data/likert/", pattern="*.csv")
survey <- do.call(rbind,lapply(
    likert_files, function(f) {
        read.csv(
            paste("/Users/br-gaster/dev/bgaster.github.io/muses_material_survey/rust_survey/assets/data/likert",f, sep="/"), 
            sep = ",", 
            row.names=NULL)
    }))

# Select the fields we care about for resulting plot
survey <- survey[,c("Category", "Gesture", "Material","Feeling","Answer")]
colnames(survey) <- c("category", "gesture", "material", "feeling", "answer")

# define the colors on the Likert scale, using the Muses color palatte
myColors <- c("#605fa4","#d989bc","#f5e5c1","#f3b73b","#dd4921","black")

# TAP gesture plot
tap_agg_table <- sqldf::sqldf(
    paste("select gesture, material, category, feeling, SUM(answer) as total 
     from survey", "where material=1 and gesture='Tap'",  "group by material, feeling, category", sep = " "))

tap_summarized_table <- tap_agg_table %>%
    group_by(material) %>%
    mutate(countT= sum(total)) %>%
    group_by(category, add=TRUE) %>%
    mutate(per=round(100*total/countT,2))

tap_summarized_table$category <- relevel(tap_summarized_table$category,"Strongly Disagree")
tap_summarized_table$category <- relevel(tap_summarized_table$category,"Disagree")
tap_summarized_table$category <- relevel(tap_summarized_table$category,"Neutral")
tap_summarized_table$category <- relevel(tap_summarized_table$category,"Agree")
tap_summarized_table$category <- relevel(tap_summarized_table$category,"Strongly Agree")

# Press gesture plot
tap_2_agg_table <- sqldf::sqldf(
    paste("select gesture, material, category, feeling, SUM(answer) as total 
     from survey", "where material=2 and gesture='Tap'",  "group by material, feeling, category", sep = " "))

tap_2_summarized_table <- tap_2_agg_table %>%
    group_by(material) %>%
    mutate(countT= sum(total)) %>%
    group_by(category, add=TRUE) %>%
    mutate(per=round(100*total/countT,2))

tap_2_summarized_table$category <- relevel(tap_2_summarized_table$category,"Strongly Disagree")
tap_2_summarized_table$category <- relevel(tap_2_summarized_table$category,"Disagree")
tap_2_summarized_table$category <- relevel(tap_2_summarized_table$category,"Neutral")
tap_2_summarized_table$category <- relevel(tap_2_summarized_table$category,"Agree")
tap_2_summarized_table$category <- relevel(tap_2_summarized_table$category,"Strongly Agree")


#actual plot creation 
tap_1_plot <- ggplot(data = tap_summarized_table, aes(x =feeling , y = per, fill = category)) +geom_bar(stat="identity", width = 0.7) +scale_fill_manual (values=myColors) +coord_flip() + ylab("") + xlab("") +theme(axis.text=element_text(size=12),axis.title=element_text(size=14,face="bold")) +ggtitle("Tap Gesture") +theme(plot.title = element_text(size = 20, face = "bold",hjust = 0.5))
tap_2_plot <- ggplot(data = tap_2_summarized_table, aes(x =feeling , y = per, fill = category)) +geom_bar(stat="identity", width = 0.7) +scale_fill_manual (values=myColors) +coord_flip() + ylab("Percentage") + xlab("") +theme(axis.text=element_text(size=12),axis.title=element_text(size=14,face="bold")) +ggtitle("") +theme(plot.title = element_text(size = 20, face = "bold",hjust = 0.5))
ggarrange(tap_1_plot, tap_2_plot, 
          labels = c("Material 1", "Material 2"),
          ncol = 1, nrow = 2)
# save plot to PDF
ggsave(file = "survey_likert_output.pdf")